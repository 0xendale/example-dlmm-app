mod mint_account;
mod pair_account;
mod pool_state;
mod token_meta;
mod types;

use std::sync::Arc;

use anyhow::Result;
use jupiter_amm_interface::KeyedAccount;
pub use mint_account::*;
pub use pool_state::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
pub use token_meta::*;
pub use types::*;

pub struct State {
    pub pool_state: Option<PoolState>,
    pub mint_accounts: Vec<MintAccount>,
}

pub trait Fetch {
    fn fetch(client: Arc<RpcClient>, key: Pubkey) -> Result<Self>
    where
        Self: Sized;
}

impl Fetch for KeyedAccount {
    fn fetch(client: Arc<RpcClient>, pair_key: Pubkey) -> Result<Self> {
        pair_account::fetch(client, pair_key)
    }
}

impl State {
    pub async fn generate_state_async(client: Arc<RpcClient>, pair_account: KeyedAccount) -> Self {
        tokio::task::spawn_blocking(move || {
            // ---- run in a separate thread, safe with runtime ----
            let state = PoolState::fetch(client.clone(), pair_account);
            let mint_x_account = MintAccount::fetch(client.clone(), state.mint_x);
            let mint_y_account = MintAccount::fetch(client.clone(), state.mint_y);

            State {
                pool_state: Some(state),
                mint_accounts: vec![mint_x_account, mint_y_account],
            }
        })
        .await
        .expect("spawn_blocking failed")
    }

    pub async fn generate_keyed_account(
        client: Arc<RpcClient>,
        pair_key: Pubkey,
    ) -> Result<KeyedAccount> {
        let pair_account = tokio::task::spawn_blocking(move || {
            // ---- run in a separate thread, safe with runtime ----
            KeyedAccount::fetch(client.clone(), pair_key)
        })
        .await
        .expect("spawn_blocking failed")?;

        Ok(pair_account)
    }

    pub async fn generate_token_state(
        client: Arc<RpcClient>,
        mint_key: Pubkey,
    ) -> Result<TokenMeta> {
        let token_state = tokio::task::spawn_blocking(move || {
            // ---- run in a separate thread, safe with runtime ----
            TokenMeta::fetch(client.clone(), mint_key)
        })
        .await
        .expect("spawn_blocking failed")?;

        Ok(token_state)
    }
}
