use anyhow::{Error, Result};
use base64::{engine::general_purpose, Engine};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, transaction::VersionedTransaction};
use solana_transaction_status_client_types::UiTransactionEncoding;

async fn simuate_tx(rpc_client: &RpcClient, tx: VersionedTransaction) -> Result<()> {
    let client = rpc_client.clone();

    // let b64_tx = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEEjNmKiZGiOtSZ+g0//wH5kEQo3+UzictY+KlLV8hjXcs44M/Xnr+1SlZsqS6cFMQc46yj9PIsxqkycxJmXT+veJjIvefX4nhY9rY+B5qreeqTHu4mG6Xtxr5udn4MN8PnBt324e51j94YQl285GzN2rYa/E2DuQ0n/r35KNihi/zamQ6EeyeeVDvPVgUO2W3Lgt9hT+CfyqHvIa11egFPCgEDAwIBAAkDZAAAAAAAAAA=";
    // let tx_bytes = general_purpose::STANDARD.decode(b64_tx).unwrap();
    // let tx: VersionedTransaction = bincode::deserialize(&tx_bytes).unwrap();

    let config = RpcSimulateTransactionConfig {
        commitment: CommitmentConfig::finalized().into(),
        encoding: UiTransactionEncoding::Base64.into(),
        replace_recent_blockhash: true,
        sig_verify: false,
        min_context_slot: None,
        inner_instructions: false,
        accounts: None,
    };

    let simulate_result = client.simulate_transaction_with_config(&tx, config).await?;

    if simulate_result.value.err.is_some() {
        return Err(Error::msg("Simulation failed"));
    } else {
        println!("Simulation succeeded!");
        println!(
            "Logs: {:?}",
            simulate_result
                .value
                .logs
                .unwrap_or_else(|| vec!["No logs".to_string()])
        );
    }
    Ok(())
}
