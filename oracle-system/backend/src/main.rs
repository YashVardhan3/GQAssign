mod pyth_client;
mod switchboard_client;
mod oracle_manager;
mod price_aggregator;
mod database;
mod api;

use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use dotenv::dotenv;
use std::env;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let pg_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

    let rpc_client = Arc::new(RpcClient::new(rpc_url));
    
    let pyth_client = Arc::new(pyth_client::PythClient::new(rpc_client.clone()));
    let switchboard_client = Arc::new(switchboard_client::SwitchboardClient::new(rpc_client.clone()));
    let aggregator = Arc::new(price_aggregator::PriceAggregator::new(100)); // 1% max deviation (100 bps)
    let database = Arc::new(database::Database::new(&pg_url, &redis_url).await?);
    
    // Run migrations
    database.migrate().await?;

    let mut oracle_manager = oracle_manager::OracleManager::new(
        pyth_client,
        switchboard_client,
        aggregator,
        database.clone(),
    );

    // Example Config - SOL/USD
    // Replace with actual feed IDs for Devnet/Mainnet
    oracle_manager.add_config(oracle_manager::OracleConfig {
        symbol: "SOL".to_string(),
        pyth_feed: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), // Devnet SOL/USD (Placeholder)
        switchboard_feed: Pubkey::from_str("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR").unwrap(), // Example
    });

    // Spawn API Server
    let db_clone = database.clone();
    tokio::spawn(async move {
        api::start_api_server(db_clone).await;
    });

    // Start Oracle Loop
    println!("Starting Oracle Service...");
    oracle_manager.start_price_loop().await;

    Ok(())
}
