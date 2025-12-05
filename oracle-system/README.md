# Oracle Integration & Price Feed System

This system provides a robust, manipulation-resistant price feed for a perpetual futures DEX on Solana. It consists of an on-chain Anchor program for validation and a Rust backend service for aggregation and caching.

## Architecture

### 1. Solana Smart Contract (Anchor)
Located in `programs/oracle-contract`.
- **Features**:
    - Validates Pyth price feeds (staleness, confidence).
    - Validates Switchboard feeds.
    - Calculates consensus on-chain if multiple prices are provided.
    - Enforces strict deviation checks.

### 2. Rust Backend Service
Located in `backend`.
- **Components**:
    - **Oracle Manager**: Orchestrates fetching from multiple sources.
    - **Pyth/Switchboard Clients**: Interfaces with Solana RPC.
    - **Price Aggregator**: Computes median price, filters outliers.
    - **Database**: PostgreSQL for history, Redis for real-time caching.
    - **API**: REST API for frontend/trading engine consumption.

## Setup

### Prerequisites
- Rust 1.75+
- Solana CLI
- PostgreSQL
- Redis

### Configuration
Create a `.env` file in `backend/`:
```
SOLANA_RPC_URL=https://api.devnet.solana.com
DATABASE_URL=postgres://user:password@localhost/oracle_db
REDIS_URL=redis://127.0.0.1/
```

### Running the Backend
```bash
cd backend
cargo run
```

### Deploying the Contract
```bash
cd programs/oracle-contract
anchor build
anchor deploy
```

## API Endpoints

- `GET /oracle/price/:symbol` - Get latest consensus price.
- `GET /oracle/health` - System health check.

## Database Schema
See `backend/migrations/001_initial_schema.sql`.
