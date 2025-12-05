use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use anyhow::Result;
use crate::price_aggregator::ConsensusPrice;
use std::sync::Arc;

pub struct Database {
    pg_pool: Pool<Postgres>,
    redis_client: redis::Client,
}

impl Database {
    pub async fn new(pg_url: &str, redis_url: &str) -> Result<Self> {
        let pg_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(pg_url)
            .await?;

        let redis_client = redis::Client::open(redis_url)?;

        Ok(Self {
            pg_pool,
            redis_client,
        })
    }

    pub async fn cache_price(&self, symbol: &str, price: &ConsensusPrice) -> Result<()> {
        let mut con = self.redis_client.get_async_connection().await?;
        let key = format!("price:{}", symbol);
        let json = serde_json::to_string(price)?;
        
        // Set with expiry (e.g., 5 seconds) to ensure we don't serve very old data if system crashes
        con.set_ex(key, json, 5).await?;
        Ok(())
    }

    pub async fn get_cached_price(&self, symbol: &str) -> Result<Option<ConsensusPrice>> {
        let mut con = self.redis_client.get_async_connection().await?;
        let key = format!("price:{}", symbol);
        let json: Option<String> = con.get(key).await?;

        match json {
            Some(j) => Ok(Some(serde_json::from_str(&j)?)),
            None => Ok(None),
        }
    }

    pub async fn save_price_history(&self, symbol: &str, price: &ConsensusPrice) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO price_history (symbol, price, confidence, timestamp, sources_used)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            symbol,
            price.price,
            price.confidence,
            chrono::NaiveDateTime::from_timestamp_opt(price.timestamp, 0),
            price.sources_used as i32
        )
        .execute(&self.pg_pool)
        .await?;
        Ok(())
    }
}
