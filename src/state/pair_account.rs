use std::sync::Arc;

use jupiter_amm_interface::{KeyedAccount, KeyedUiAccount};
use serde::{Deserialize, Serialize};
use solana_account_decoder::{encode_ui_account, UiAccountEncoding};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};

use anyhow::Result;

#[derive(Clone, Deserialize, Serialize)]
pub struct PairAccount {
    pub key: Pubkey,
    pub account: Account,
    pub keyed_account: KeyedAccount,
}

impl PairAccount {
    pub fn _fetch(client: Arc<RpcClient>, pair_key: Pubkey) -> Result<PairAccount> {
        let account = client.get_account(&pair_key)?;

        let ui_account =
            encode_ui_account(&pair_key, &account, UiAccountEncoding::Base64, None, None);

        let keyed_ui_account = KeyedUiAccount {
            pubkey: pair_key.to_string(),
            ui_account,
            params: None,
        };

        let keyed_account: KeyedAccount = keyed_ui_account.try_into()?;

        Ok(PairAccount {
            key: pair_key,
            account,
            keyed_account,
        })
    }

    pub fn _into_keyed_account(self) -> KeyedAccount {
        KeyedAccount {
            key: self.key,
            account: self.account,
            params: None,
        }
    }
}

pub fn fetch(client: Arc<RpcClient>, pair_key: Pubkey) -> Result<KeyedAccount> {
    let account = client.get_account(&pair_key)?;

    Ok(KeyedAccount {
        key: pair_key,
        account,
        params: None,
    })
}
