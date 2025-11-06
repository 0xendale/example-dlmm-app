use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use ahash::RandomState;
use jupiter_amm_interface::{Amm, Quote, QuoteParams};
use saros_dlmm::SarosDlmm;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::app::AppContext;

pub struct DLMMClient {
    pub saros_dlmm: Arc<RwLock<SarosDlmm>>,
}

pub trait UpdateAmm: Amm {
    async fn update_amm(&mut self, ctx: &AppContext) -> Result<()>;
}

impl UpdateAmm for SarosDlmm {
    async fn update_amm(&mut self, ctx: &AppContext) -> Result<()> {
        let accounts_to_update = self.get_accounts_to_update();
        let account_map: HashMap<Pubkey, Account, RandomState> = ctx
            .rpc_client
            .get_multiple_accounts(&accounts_to_update)
            .unwrap()
            .into_iter()
            .zip(accounts_to_update)
            .fold(HashMap::default(), |mut m, (account, address)| {
                if let Some(account) = account {
                    m.insert(address, account);
                }
                m
            });
        self.update(&account_map).unwrap();

        Ok(())
    }
}

impl DLMMClient {
    pub async fn update(&self, ctx: &AppContext) -> Result<()> {
        let mut s = self.saros_dlmm.write().await;
        s.update_amm(ctx).await; // ✅ mutate safely
        Ok(())
    }

    pub async fn quote(&self, quote_params: QuoteParams) -> Result<Quote> {
        let s = self.saros_dlmm.read().await;
        let quote_result = s.quote(&quote_params)?; // ✅ read safely
        Ok(quote_result)
    }
}
