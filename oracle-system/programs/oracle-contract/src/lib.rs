use anchor_lang::prelude::*;
use pyth_sdk_solana::load_price_feed_from_account_info;
use std::cmp::Ordering;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod oracle_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, symbol: String, max_staleness: i64, max_confidence: u64, max_deviation: u64) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        oracle_config.symbol = symbol;
        oracle_config.max_staleness = max_staleness;
        oracle_config.max_confidence = max_confidence;
        oracle_config.max_deviation = max_deviation;
        oracle_config.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn get_pyth_price(
        ctx: Context<GetPythPrice>,
    ) -> Result<PriceData> {
        let oracle_config = &ctx.accounts.oracle_config;
        let price_feed_info = &ctx.accounts.price_feed;
        
        let price_feed = load_price_feed_from_account_info(price_feed_info)
            .map_err(|_| ErrorCode::InvalidPythFeed)?;

        let current_price = price_feed.get_price_no_older_than(oracle_config.max_staleness, Clock::get()?.unix_timestamp)
            .ok_or(ErrorCode::StalePrice)?;

        // Check confidence interval
        // Confidence is absolute value in same units as price. 
        // We want to check if confidence / price > max_confidence (in basis points)
        // conf * 10000 / price <= max_confidence
        let conf_bps = (current_price.conf as u128)
            .checked_mul(10000)
            .unwrap()
            .checked_div(current_price.price.abs() as u128)
            .unwrap_or(u128::MAX);

        if conf_bps > oracle_config.max_confidence as u128 {
            return err!(ErrorCode::LowConfidence);
        }

        Ok(PriceData {
            price: current_price.price,
            confidence: current_price.conf,
            expo: current_price.expo,
            timestamp: current_price.publish_time,
            source: PriceSource::Pyth,
        })
    }

    // Placeholder for Switchboard - actual implementation depends on specific Switchboard version crates
    pub fn get_switchboard_price(
        ctx: Context<GetSwitchboardPrice>,
    ) -> Result<PriceData> {
        // In a real implementation, we would deserialize the Switchboard Aggregator account here
        // For this example, we'll simulate reading it or assume a wrapper
        
        // let aggregator = &ctx.accounts.aggregator;
        // let val: f64 = aggregator.get_result()?;
        // ... validation logic ...

        // Returning dummy data for compilation as Switchboard crate specifics vary heavily by version
        Ok(PriceData {
            price: 100000000,
            confidence: 1000,
            expo: -6,
            timestamp: Clock::get()?.unix_timestamp,
            source: PriceSource::Switchboard,
        })
    }

    pub fn validate_price_consensus(
        ctx: Context<ValidatePrice>,
        prices: Vec<PriceData>,
    ) -> Result<i64> {
        let oracle_config = &ctx.accounts.oracle_config;
        
        if prices.is_empty() {
            return err!(ErrorCode::NoPricesProvided);
        }

        // Sort prices to find median
        let mut sorted_prices = prices.clone();
        sorted_prices.sort_by(|a, b| {
            // Normalize prices to same exponent for comparison if needed, 
            // for simplicity assuming same exponent or normalized before passing
            a.price.cmp(&b.price)
        });

        let mid = sorted_prices.len() / 2;
        let median_price = if sorted_prices.len() % 2 == 0 {
            (sorted_prices[mid - 1].price + sorted_prices[mid].price) / 2
        } else {
            sorted_prices[mid].price
        };

        // Validate deviation
        for price_data in prices.iter() {
            let deviation = (price_data.price - median_price).abs();
            let deviation_bps = (deviation as u128)
                .checked_mul(10000)
                .unwrap()
                .checked_div(median_price.abs() as u128)
                .unwrap_or(0);

            if deviation_bps > oracle_config.max_deviation as u128 {
                // Log or reject specific outlier? 
                // For strict consensus, if any trusted source is too far off, we might want to fail or just exclude it.
                // Here we fail for safety.
                return err!(ErrorCode::PriceDeviationTooHigh);
            }
        }

        Ok(median_price)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 32)]
    pub oracle_config: Account<'info, OracleConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPythPrice<'info> {
    pub oracle_config: Account<'info, OracleConfig>,
    /// CHECK: We check the account owner and data in the instruction logic using Pyth SDK
    pub price_feed: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetSwitchboardPrice<'info> {
    pub oracle_config: Account<'info, OracleConfig>,
    /// CHECK: Switchboard aggregator account
    pub aggregator: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ValidatePrice<'info> {
    pub oracle_config: Account<'info, OracleConfig>,
}

#[account]
pub struct OracleConfig {
    pub symbol: String,
    pub authority: Pubkey,
    pub max_staleness: i64,  // seconds
    pub max_confidence: u64,  // basis points
    pub max_deviation: u64,   // basis points
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceData {
    pub price: i64,
    pub confidence: u64,
    pub expo: i32,
    pub timestamp: i64,
    pub source: PriceSource,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum PriceSource {
    Pyth,
    Switchboard,
    Internal,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Pyth price feed is invalid")]
    InvalidPythFeed,
    #[msg("Price is stale")]
    StalePrice,
    #[msg("Confidence interval is too low")]
    LowConfidence,
    #[msg("No prices provided for consensus")]
    NoPricesProvided,
    #[msg("Price deviation exceeds threshold")]
    PriceDeviationTooHigh,
}
