use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePrice {
    pub price: f64,
    pub confidence: f64,
    pub timestamp: i64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusPrice {
    pub price: f64,
    pub confidence: f64,
    pub timestamp: i64,
    pub sources_used: usize,
}

pub struct PriceAggregator {
    max_deviation_bps: u64,
}

impl PriceAggregator {
    pub fn new(max_deviation_bps: u64) -> Self {
        Self { max_deviation_bps }
    }

    pub fn calculate_consensus(&self, mut prices: Vec<SourcePrice>) -> Result<ConsensusPrice> {
        if prices.is_empty() {
            return Err(anyhow!("No prices to aggregate"));
        }

        // Filter out stale prices (e.g., > 30 seconds old)
        let now = chrono::Utc::now().timestamp();
        prices.retain(|p| now - p.timestamp < 30);

        if prices.is_empty() {
            return Err(anyhow!("All prices are stale"));
        }

        // Sort by price
        prices.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        // Median calculation
        let mid = prices.len() / 2;
        let median_price = if prices.len() % 2 == 0 {
            (prices[mid - 1].price + prices[mid].price) / 2.0
        } else {
            prices[mid].price
        };

        // Check deviation
        let mut valid_prices = Vec::new();
        for p in prices {
            let deviation = (p.price - median_price).abs();
            let deviation_bps = (deviation / median_price * 10000.0) as u64;

            if deviation_bps <= self.max_deviation_bps {
                valid_prices.push(p);
            }
        }

        if valid_prices.is_empty() {
            return Err(anyhow!("No prices within deviation threshold"));
        }

        // Recalculate median of valid prices or use weighted average based on confidence
        // For simplicity, using average of valid prices here
        let sum_price: f64 = valid_prices.iter().map(|p| p.price).sum();
        let avg_price = sum_price / valid_prices.len() as f64;

        // Conservative confidence: max confidence interval of used sources
        let max_conf = valid_prices.iter().map(|p| p.confidence).fold(0.0, f64::max);

        Ok(ConsensusPrice {
            price: avg_price,
            confidence: max_conf,
            timestamp: now,
            sources_used: valid_prices.len(),
        })
    }
}
