CREATE TABLE IF NOT EXISTS price_history (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    sources_used INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_price_history_symbol_timestamp ON price_history(symbol, timestamp DESC);
