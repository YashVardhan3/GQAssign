use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use pyth_sdk_solana::load_price_feed_from_account;
use anyhow::{Result, anyhow};
use std::sync::Arc;

pub struct PythClient {
    rpc_client: Arc<RpcClient>,
}

impl PythClient {
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self {
            rpc_client,
        }
    }

    pub async fn get_price(&self, price_feed_id: &Pubkey) -> Result<f64> {
        let client = self.rpc_client.clone();
        let feed_id = *price_feed_id;

        let mut account = tokio::task::spawn_blocking(move || {
            client.get_account(&feed_id)
        }).await??;

        let price_feed = load_price_feed_from_account(&feed_id, &mut account)
            .map_err(|e| anyhow!("Failed to load Pyth feed: {:?}", e))?;

        let current_price = price_feed.get_price_unchecked();

        let price = current_price.price as f64 * 10f64.powi(current_price.expo);
        Ok(price)
    }

    pub async fn get_price_data(&self, price_feed_id: &Pubkey) -> Result<(f64, f64, i64)> {
        let client = self.rpc_client.clone();
        let feed_id = *price_feed_id;

        let mut account = tokio::task::spawn_blocking(move || {
            client.get_account(&feed_id)
        }).await??;

        let price_feed = load_price_feed_from_account(&feed_id, &mut account)
            .map_err(|e| anyhow!("Failed to load Pyth feed: {:?}", e))?;

        let current_price = price_feed.get_price_unchecked();

        let price = current_price.price as f64 * 10f64.powi(current_price.expo);
        let conf = current_price.conf as f64 * 10f64.powi(current_price.expo);
        
        Ok((price, conf, current_price.publish_time))
    }
}
