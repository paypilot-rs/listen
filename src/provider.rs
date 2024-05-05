use std::{f64::consts::E, str::FromStr};

use crate::{constants, types};

use solana_client::{
    rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
    rpc_request::TokenAccountsFilter,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
};
use spl_token::state::Account;

pub fn get_client(url: &str) -> Result<RpcClient, Box<dyn std::error::Error>> {
    let rpc_client =
        RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let identity = rpc_client.get_identity()?;

    println!("Connecting to blocks through {:?}", url);
    println!("Latest blockhash: {:?}", latest_blockhash);
    println!("Identity: {:?}", identity);

    Ok(rpc_client)
}

// Provider provides the data, contains both RPC client that can
// communicate over the REST interface and utilities like getting
// the pricing data from Jupiter
pub struct Provider {
    rpc_client: RpcClient,
}

impl Provider {
    pub fn new(rpc_url: String) -> Provider {
        Provider {
            rpc_client: get_client(rpc_url.as_str()).unwrap(),
        }
    }

    pub fn get_balance(
        &self,
        pubkey: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let balance = self.rpc_client.get_balance(pubkey)?;
        Ok(balance)
    }

    pub fn get_spl_balance(
        &self,
        pubkey: &Pubkey,
        mint: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let token_accounts = self.rpc_client.get_token_accounts_by_owner(
            pubkey,
            TokenAccountsFilter::Mint(*mint),
        )?;
        for token_account in token_accounts {
            let acount_info = self.rpc_client.get_account(
                &Pubkey::from_str(token_account.pubkey.as_str())?,
            )?;
            let token_account_info = Account::unpack(&acount_info.data)?;
            println!("Token account info: {:?}", token_account_info);
            return Ok(token_account_info.amount);
        }
        Err("No token account found".into())
    }

    pub fn get_tx(
        &self,
        signature: &str,
    ) -> Result<
        EncodedConfirmedTransactionWithStatusMeta,
        Box<dyn std::error::Error>,
    > {
        let sig = Signature::from_str(signature)?;
        let tx = self.rpc_client.get_transaction_with_config(
            &sig,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(1),
            },
        )?;
        Ok(tx)
    }

    pub async fn get_pricing(
        &self,
        mint: &str,
    ) -> Result<types::PriceResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://price.jup.ag/v4/price?ids={}&vsToken={}",
            mint,
            constants::SOLANA_PROGRAM_ID,
        );
        println!("Getting pricing from: {:?}", url);
        let client = reqwest::Client::new();
        let res = client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await?;
        let data = res.json::<types::PriceResponse>().await?;
        Ok(data)
    }

    pub fn send_tx(
        &self,
        tx: &solana_sdk::transaction::VersionedTransaction,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self
            .rpc_client
            .send_and_confirm_transaction_with_spinner(tx)
        {
            Ok(signature) => {
                return Ok(signature.to_string());
            }
            Err(e) => {
                return Err(e.into());
            }
        };
    }
}
