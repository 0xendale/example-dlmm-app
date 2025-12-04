use anchor_lang::prelude::AccountMeta;
use saros_sdk::{
    instruction::build_swap_instruction_data, math::swap_manager::SwapType,
    utils::helper::is_swap_for_y,
};
use solana_client::{client_error::reqwest, rpc_request::RpcRequest};
use solana_program::example_mocks::solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    transaction::{Transaction, TransactionVersion},
};
use spl_token_metadata_interface::solana_instruction::Instruction as SolInstruction;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use tracing::info;

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::{
    app::{AppConfig, AppContext},
    state::{
        InstructionRequest, InstructionType, QuoteRequest, Status, SwapInstructionParams,
        WebJsonResponse,
    },
};
use anyhow::Result;
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
};

use jupiter_amm_interface::{Amm, QuoteParams, SwapMode, SwapParams};

use base64::{engine::general_purpose, Engine as _};

pub async fn start_web_server(config: AppConfig) -> Result<()> {
    let app_state = Arc::new(AppContext::new(config));

    let static_files = ServeDir::new(format!("{}/web/dist", env!("CARGO_MANIFEST_DIR")));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new().route("/api/network/status", get(ping));

    let sdk_routes = Router::new()
        .route("/api/pair", get(get_pair))
        .route("/api/quote", post(get_quote))
        .route("/api/instruction", post(get_instruction))
        .route("/api/simulate_swap", post(simulate_swap));

    // Define API routes
    let app = Router::new()
        .merge(public_routes)
        .merge(sdk_routes)
        .route("/api/ping", get(|| async { "pong 🦀" }))
        .fallback_service(static_files)
        .layer(cors)
        .with_state(app_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// === Handlers ===
async fn ping() -> &'static str {
    "pong 🦀"
}

/// Get pool info by pubkey
#[axum::debug_handler]
async fn get_pair(
    State(ctx): State<Arc<AppContext>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<WebJsonResponse> {
    let pair_address = params.get("address").cloned().unwrap_or_default();
    if pair_address.len() < 20 {
        return Json(WebJsonResponse {
            status: Status::Error,
            message: "Invalid address format".to_string(),
            data: json!({}),
        });
    }

    let pair_key = Pubkey::from_str_const(&pair_address);

    // Step 1: Get or create DLMM client
    let dlmm_client = match ctx.get_or_spawn_client(pair_key).await {
        Ok(client) => client,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to get DLMM client: {}", e),
                data: json!({}),
            });
        }
    };

    info!("🔍 Fetching metadata from RPC for pair {}", pair_address);
    let saros_dlmm = dlmm_client.saros_dlmm.read().await;

    let [mint_a_meta, mint_b_meta] = match ctx.fetch_pair_token_info(&saros_dlmm).await {
        Ok(mints) => mints,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to fetch token metadata: {}", e),
                data: json!({}),
            });
        }
    };

    Json(WebJsonResponse {
        status: Status::Success,
        message: "Pair fetched successfully".to_string(),
        data: json!({
            "pair_address": pair_address,
            "token_mint_x": saros_dlmm.pair.token_mint_x.to_string(),
            "token_mint_y": saros_dlmm.pair.token_mint_y.to_string(),
            "token_a": {
                "mint": mint_a_meta.mint.to_string(),
                "symbol": mint_a_meta.symbol,
                "decimals": mint_a_meta.decimals,
            },
            "token_b": {
                "mint": mint_b_meta.mint.to_string(),
                "symbol": mint_b_meta.symbol,
                "decimals": mint_b_meta.decimals,
            },
        }),
    })
}

#[axum::debug_handler]
async fn get_quote(
    State(ctx): State<Arc<AppContext>>,
    Json(body): Json<QuoteRequest>,
) -> Json<WebJsonResponse> {
    let pair_address = body.pair_address.clone();

    info!("🔍 Getting quote for pair {}", pair_address);
    info!("Body: {:?}", body);

    let source_mint = Pubkey::from_str_const(&body.source_mint);
    let destination_mint = Pubkey::from_str_const(&body.destination_mint);

    // 1️⃣ take DLMM client
    let dlmm_client = match ctx
        .get_or_spawn_client(Pubkey::from_str_const(&pair_address))
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to get DLMM client: {}", e),
                data: json!({}),
            });
        }
    };

    tracing::info!(
        "💱 Quoting swap: amount_in={}, source_mint={}",
        body.amount_in,
        body.source_mint
    );

    for _ in 0..3 {
        if let Err(e) = dlmm_client.update(&ctx).await {
            tracing::warn!("⚠️ Failed to update DLMM client: {}", e);
            continue;
        }
    }

    let client = dlmm_client.saros_dlmm.read().await;

    let is_swap_for_y = is_swap_for_y(source_mint, client.pair.token_mint_x);
    let swap_mode = if is_swap_for_y {
        if source_mint == client.pair.token_mint_x {
            SwapMode::ExactIn
        } else {
            SwapMode::ExactOut
        }
    } else {
        if source_mint == client.pair.token_mint_x {
            SwapMode::ExactOut
        } else {
            SwapMode::ExactIn
        }
    };

    let req = QuoteParams {
        amount: body.amount_in,
        input_mint: source_mint,
        swap_mode,
        output_mint: destination_mint,
    };

    // 2️⃣ call get_quote() from DLMM client
    let result = match client.quote(&req) {
        Ok(quote) => Json(WebJsonResponse {
            status: Status::Success,
            message: "quote successful".to_string(),
            data: json!({
                "in_amount": quote.in_amount,
                "out_amount": quote.out_amount,
                "fee_amount": quote.fee_amount,
                "fee_mint": quote.fee_mint.to_string(),
            }),
        }),
        Err(e) => Json(WebJsonResponse {
            status: Status::Error,
            message: format!("Failed to get quote: {}", e),
            data: json!({}),
        }),
    };

    result
}

#[axum::debug_handler]
async fn get_instruction(
    State(ctx): State<Arc<AppContext>>,
    Json(body): Json<InstructionRequest<serde_json::Value>>,
) -> Json<WebJsonResponse> {
    let pair_address = body.pair_address.clone();

    let params: SwapInstructionParams = serde_json::from_value(body.params.clone()).unwrap();
    info!("🔍 Getting instruction for pair {}", pair_address);
    let source_mint = Pubkey::from_str_const(&params.source_mint);
    let destination_mint = Pubkey::from_str_const(&params.destination_mint);

    // 1️⃣ take DLMM client
    let dlmm_client = match ctx
        .get_or_spawn_client(Pubkey::from_str_const(&pair_address))
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to get DLMM client: {}", e),
                data: json!({}),
            });
        }
    };

    let client = dlmm_client.saros_dlmm.read().await;

    let is_swap_for_y = is_swap_for_y(source_mint, client.pair.token_mint_x);
    let swap_mode = if is_swap_for_y {
        if source_mint == client.pair.token_mint_x {
            SwapMode::ExactIn
        } else {
            SwapMode::ExactOut
        }
    } else {
        if source_mint == client.pair.token_mint_x {
            SwapMode::ExactOut
        } else {
            SwapMode::ExactIn
        }
    };

    let (instruction_type, params) = match body.instruction_type {
        InstructionType::Swap => {
            let params: SwapInstructionParams = serde_json::from_value(body.params).unwrap();
            info!("💱 Generating swap instruction with params: {:?}", params);

            (
                "swap",
                json!({
                    "amount_in": params.in_amount,
                    "minimum_amount_out": params.min_out_amount,
                    "source_mint": params.source_mint,
                    "destination_mint": params.destination_mint,
                }),
            )
        }
        _ => ("unsupported", json!({})),
        // InstructionType::ClosePosition => ("close_position", json!({})),
    };

    // Implementation for fetching instruction details goes here
    Json(WebJsonResponse {
        status: Status::Success,
        message: "Instruction fetched successfully".to_string(),
        data: json!({}),
    })
}

#[axum::debug_handler]
async fn simulate_swap(
    State(ctx): State<Arc<AppContext>>,
    Json(body): Json<InstructionRequest<serde_json::Value>>,
) -> Json<WebJsonResponse> {
    info!("🔍 Simulating swap with body: {:?}", body);

    let pair_address = body.pair_address.clone();
    let params: SwapInstructionParams = serde_json::from_value(body.params.clone()).unwrap();

    info!("🔍 Getting instruction for pair {}", pair_address);

    let source_mint = Pubkey::from_str_const(&params.source_mint);
    let destination_mint = Pubkey::from_str_const(&params.destination_mint);
    let in_amount = params.in_amount;
    let min_out_amount = params.min_out_amount;

    // 1️⃣ take DLMM client
    let dlmm_client = match ctx
        .get_or_spawn_client(Pubkey::from_str_const(&pair_address))
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to get DLMM client: {}", e),
                data: json!({}),
            });
        }
    };

    for _ in 0..3 {
        if let Err(e) = dlmm_client.update(&ctx).await {
            tracing::warn!("⚠️ Failed to update DLMM client: {}", e);
            continue;
        }
    }

    let client = dlmm_client.saros_dlmm.read().await;

    let user = Pubkey::from_str_const(&params.signer);

    // let source_token_account = get_associated_token_address(&user, &source_mint).to_string();
    let user_token_vault_x = get_associated_token_address_with_program_id(
        &user,
        &client.pair.token_mint_x,
        &client.token_program[0],
    );

    let user_token_vault_y = get_associated_token_address_with_program_id(
        &user,
        &client.pair.token_mint_y,
        &client.token_program[1],
    );

    let is_swap_for_y = is_swap_for_y(source_mint, client.pair.token_mint_x);
    let swap_mode = if is_swap_for_y {
        if source_mint == client.pair.token_mint_x {
            SwapType::ExactIn
        } else {
            SwapType::ExactOut
        }
    } else {
        if source_mint == client.pair.token_mint_x {
            SwapType::ExactOut
        } else {
            SwapType::ExactIn
        }
    };

    let bin_for_swap = client.compute_bin_array_swap().unwrap();

    let (instruction_type, params) = match body.instruction_type {
        InstructionType::Swap => {
            let params: SwapInstructionParams = serde_json::from_value(body.params).unwrap();

            // let fake_hash = Hash::new_unique();

            let swap_instruction_data = build_swap_instruction_data(
                saros_sdk::instruction::BuildSwapInstructionDataParams {
                    amount: in_amount,
                    other_amount_threshold: min_out_amount,
                    swap_for_y: is_swap_for_y,
                    swap_mode,
                },
            )
            .unwrap();

            let mut account_metas = Vec::new();

            {
                account_metas.push(AccountMeta::new(client.key, false));
                account_metas.push(AccountMeta::new_readonly(client.pair.token_mint_x, false));
                account_metas.push(AccountMeta::new_readonly(client.pair.token_mint_y, false));
                account_metas.push(AccountMeta::new(bin_for_swap.bin_array_keys[0], false));
                account_metas.push(AccountMeta::new(bin_for_swap.bin_array_keys[1], false));
                account_metas.push(AccountMeta::new(client.token_vault[0], false));
                account_metas.push(AccountMeta::new(client.token_vault[1], false));
                account_metas.push(AccountMeta::new(
                    Pubkey::from_str_const(&user_token_vault_x.to_string()),
                    false,
                ));
                account_metas.push(AccountMeta::new(
                    Pubkey::from_str_const(&user_token_vault_y.to_string()),
                    false,
                ));
                account_metas.push(AccountMeta::new_readonly(user, true));
                account_metas.push(AccountMeta::new_readonly(client.token_program[0], false));
                account_metas.push(AccountMeta::new_readonly(client.token_program[1], false));
                account_metas.push(AccountMeta::new_readonly(
                    Pubkey::from_str_const("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                    false,
                ));
            }

            // If pair does not have hook, hook should be pair key (dummy)
            account_metas.push(AccountMeta::new(client.hook, false));
            account_metas.push(AccountMeta::new_readonly(ctx.config.hook_program_id, false));
            // This expect as the last of swap instruction
            account_metas.push(AccountMeta::new_readonly(client.event_authority, false));
            account_metas.push(AccountMeta::new_readonly(client.program_id, false));

            // Remaining accounts for hook CPI call
            if client.hook != client.key {
                let bin_array_index = client.pair.bin_array_index();
                let (hook_bin_array_lower, _) = Pubkey::find_program_address(
                    &[
                        b"bin_array".as_ref(),
                        client.hook.as_ref(),
                        (bin_array_index).to_le_bytes().as_ref(),
                    ],
                    &ctx.config.hook_program_id,
                );

                let (hook_bin_array_upper, _) = Pubkey::find_program_address(
                    &[
                        b"bin_array".as_ref(),
                        client.hook.as_ref(),
                        (bin_array_index + 1).to_le_bytes().as_ref(),
                    ],
                    &ctx.config.hook_program_id,
                );

                account_metas.push(AccountMeta::new(hook_bin_array_lower, false));
                account_metas.push(AccountMeta::new(hook_bin_array_upper, false));
            }

            let swap_instruction = Instruction {
                program_id: client.program_id,
                accounts: account_metas,
                data: swap_instruction_data,
            };

            println!("Swap instruction: {:?}", swap_instruction);

            let message = Message::new(&[swap_instruction], Some(&user));

            let dummy_signature = Signature::new_unique();

            let tx: Transaction = Transaction {
                signatures: vec![dummy_signature],
                message,
            };

            // 5. Serialize + base64
            let serialized = bincode::serialize(&tx).unwrap();
            let tx_b64 = general_purpose::STANDARD.encode(serialized);

            let request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "simulateTransaction",
                "params": [
                    tx_b64,
                    {
                        "encoding": "base64",
                        "sigVerify": false,
                        "replaceRecentBlockhash": true
                    }
                ]
            });

            let client = reqwest::Client::new();

            let res = client
                .post(ctx.rpc_client.url())
                .json(&request)
                .send()
                .await;

            let response: serde_json::Value = match res {
                Ok(resp) => resp.json().await.unwrap(),
                Err(e) => {
                    return Json(WebJsonResponse {
                        status: Status::Error,
                        message: format!("Failed to simulate transaction: {}", e),
                        data: json!({}),
                    });
                }
            };

            let parsed = parse_simulation_result(&response);

            (
                "swap",
                json!({
                   "response": parsed
                }),
            )
        }
        _ => ("unsupported", json!({})),
    };

    return Json(WebJsonResponse {
        status: Status::Success,
        message: "Simulation successful".to_string(),
        data: params,
    });
}

pub fn parse_simulation_result(v: &Value) -> serde_json::Value {
    let value = &v["result"]["value"];

    let logs = value["logs"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();

    // Extract error (if any)
    let error = if value["err"].is_null() {
        None
    } else {
        Some(format!("{:?}", value["err"]))
    };

    let slot = v["result"]["context"]["slot"].as_u64().unwrap_or(0);
    let fee = value["fee"].as_u64().unwrap_or(0);
    let units = value["unitsConsumed"].as_u64().unwrap_or(0);

    let pre = value["preTokenBalances"].clone();
    let post = value["postTokenBalances"].clone();

    serde_json::json!({
        "slot": slot,
        "status": if error.is_some() { "error" } else { "success" },
        "fee": fee,
        "units": units,
        "error": error,
        "logs": logs,
        "preTokenBalances": pre,
        "postTokenBalances": post
    })
}
