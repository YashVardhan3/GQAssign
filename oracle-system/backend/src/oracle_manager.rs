use crate::pyth_client::PythClient;
use crate::switchboard_client::SwitchboardClient;
use crate::price_aggregator::PriceAggregator;
use crate::database::Database;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

#[derive(Clone)]
pub struct OracleConfig {
    pub symbol: String,
    pub pyth_feed: Pubkey,
    pub switchboard_feed: Pubkey,
}

pub struct OracleManager {
    pyth_client: Arc<PythClient>,
    switchboard_client: Arc<SwitchboardClient>,
    aggregator: Arc<PriceAggregator>,
    database: Arc<Database>,
    configs: HashMap<String, OracleConfig>,
}

impl OracleManager {
    pub fn new(
        pyth_client: Arc<PythClient>,
        switchboard_client: Arc<SwitchboardClient>,
        aggregator: Arc<PriceAggregator>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            pyth_client,
            switchboard_client,
            aggregator,
            database,
            configs: HashMap::new(),
        }
    }

    pub fn add_config(&mut self, config: OracleConfig) {
        self.configs.insert(config.symbol.clone(), config);
    }

    pub async fn start_price_loop(&self) {
        let interval_duration = std::time::Duration::from_millis(500); // Sub-second updates
        let mut interval = tokio::time::interval(interval_duration);

        loop {
            interval.tick().await;
            for (symbol, config) in &self.configs {
                self.update_price(symbol, config).await;
            }
        }
    }

    async fn update_price(&self, symbol: &str, config: &OracleConfig) {
        // Fetch from Pyth
        let pyth_res = self.pyth_client.get_price_data(&config.pyth_feed).await;
        
        // Fetch from Switchboard
        let sb_res = self.switchboard_client.get_price_data(&config.switchboard_feed).await;

        let mut prices = Vec::new();

        if let Ok((price, conf, ts)) = pyth_res {
            prices.push(crate::price_aggregator::SourcePrice {
                price,
                confidence: conf,
                timestamp: ts,
                source: "Pyth".to_string(),
            });
        } else {
            error!("Failed to fetch Pyth price for {}", symbol);
        }

        if let Ok((price, conf, ts)) = sb_res {
            prices.push(crate::price_aggregator::SourcePrice {
                price,
                confidence: conf,
                timestamp: ts,
                source: "Switchboard".to_string(),
            });
        } else {
            error!("Failed to fetch Switchboard price for {}", symbol);
        }

        if prices.is_empty() {
            error!("No price sources available for {}", symbol);
            return;
        }

        // Aggregate
        match self.aggregator.calculate_consensus(prices.clone()) {
            Ok(consensus_price) => {
                info!("Updated price for {}: {}", symbol, consensus_price.price);
                
                // Cache in Redis
                if let Err(e) = self.database.cache_price(symbol, &consensus_price).await {
                    error!("Failed to cache price: {}", e);
                }

                // Store history in DB (maybe less frequently or sampled)
                if let Err(e) = self.database.save_price_history(symbol, &consensus_price).await {
                    error!("Failed to save history: {}", e);
                }
            }
            Err(e) => {
                error!("Consensus failed for {}: {}", symbol, e);
            }
        }
    }
}
