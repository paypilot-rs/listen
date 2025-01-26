pub mod config;
pub mod kv_store;
pub mod types;
pub mod util;

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use solana_sdk::transaction::Transaction;

use config::PrivyConfig;
use types::{
    CreateWalletRequest, CreateWalletResponse, PrivyClaims,
    SignAndSendTransactionParams, SignAndSendTransactionRequest,
    SignAndSendTransactionResponse, User,
};

use util::{create_http_client, transaction_to_base64};

pub struct WalletManager {
    privy_config: PrivyConfig,
    http_client: reqwest::Client,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct UserSession {
    pub(crate) user_id: String,
    pub(crate) session_id: String,
    pub(crate) wallet_address: String,
}

/// WalletManager currently only supports Solana
impl WalletManager {
    pub fn new(privy_config: PrivyConfig) -> Self {
        let http_client = create_http_client(&privy_config);
        Self {
            privy_config,
            http_client,
        }
    }

    pub async fn create_wallet(&self) -> Result<CreateWalletResponse> {
        let request = CreateWalletRequest {
            chain_type: "solana".to_string(),
        };

        let response = self
            .http_client
            .post("https://api.privy.io/v1/wallets")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to create wallet: {} - {}",
                response.status(),
                response.text().await?
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn authenticate_user(
        &self,
        access_token: &str,
    ) -> Result<UserSession> {
        let claims = self.validate_access_token(access_token)?;
        let user = self.get_user_by_id(&claims.user_id).await?;
        let wallet = user
            .linked_accounts
            .iter()
            .find_map(|account| match account {
                types::LinkedAccount::Wallet(wallet) => {
                    if wallet.delegated
                        && wallet.chain_type == "solana"
                        && wallet.wallet_client == "privy"
                    {
                        Some(wallet)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .ok_or_else(|| anyhow!("Could not find a delegated wallet"))?;

        Ok(UserSession {
            user_id: user.id,
            session_id: claims.session_id,
            wallet_address: wallet.public_key.clone(),
        })
    }

    pub async fn sign_and_send_transaction(
        &self,
        session: &UserSession,
        transaction: Transaction,
    ) -> Result<String> {
        let request = SignAndSendTransactionRequest {
            address: session.wallet_address.clone(),
            chain_type: "solana".to_string(),
            method: "signAndSendTransaction".to_string(),
            caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
            params: SignAndSendTransactionParams {
                transaction: transaction_to_base64(transaction)?,
                encoding: "base64".to_string(),
            },
        };

        let response = self
            .http_client
            .post("https://api.privy.io/v1/wallets/rpc")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to sign transaction: {}",
                response.text().await?
            ));
        }

        let result: SignAndSendTransactionResponse = response.json().await?;
        Ok(result.data.hash)
    }

    pub fn validate_access_token(
        &self,
        access_token: &str,
    ) -> Result<PrivyClaims> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&["privy.io"]);
        validation.set_audience(&[self.privy_config.app_id.clone()]);

        let key = DecodingKey::from_ec_pem(
            self.privy_config.verification_key.as_bytes(),
        )?;

        let token_data =
            decode::<PrivyClaims>(access_token, &key, &validation)
                .map_err(|_| anyhow!("Failed to authenticate"))?;

        Ok(token_data.claims)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User> {
        let url = format!("https://auth.privy.io/api/v1/users/{}", user_id);

        let response = self.http_client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get user data: {}",
                response.status()
            ));
        }

        Ok(response.json().await?)
    }
}
