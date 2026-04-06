# Excentra Exchange

A production-grade centralized cryptocurrency exchange backend built in Rust. Features a custom in-memory matching engine, real-time WebSocket streaming, JWT authentication, PostgreSQL persistence, and live price seeding via CoinGecko.

> **Status:** Phases 1–7 complete. Ethereum Sepolia testnet integration planned.

---

## Overview

Excentra models how real exchanges like Binance or Kraken work under the hood — user accounts, live order books, and a matching engine that fills buy and sell orders in real time using price-time priority (FIFO).

**Supported pairs:** BTC/USDT · ETH/USDT · SOL/USDT

**Core capabilities:**
- Limit and market order placement with immediate matching
- Real-time order book, trade feed, and 24h ticker via WebSocket
- Balance management with holds — funds locked on order placement, released on cancel
- JWT authentication with role-based access control
- Order book seeded with live prices from CoinGecko at startup

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, edition 2024)
- [PostgreSQL](https://www.postgresql.org/) (v14+)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

---

### 1. Clone the repo
```bash
git clone https://github.com/manlikeHB/excentra.git
cd excentra
```

---

### 2. Configure environment

Copy the example and fill in your values:
```bash
cp .env.example .env
```

`.env`:
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/excentra
JWT_SECRET=your-secret-key-here
API_VERSION=v1
PORT=3000
RUST_LOG=info,tower_http=debug
```

> **Note:** The server will panic at startup if any of these are missing — this is intentional.

---

### 3. Set up the database
```bash
# Create the database
sqlx database create

# Run all migrations
sqlx migrate run
```

This creates all tables and seeds BTC/USDT, ETH/USDT, and SOL/USDT trading pairs.

---

### 4. Run the server
```bash
cargo run
```

On startup, the server will:
- Rebuild the in-memory order book from open orders in the database
- Fetch live prices from CoinGecko and seed the order book
- Start the ticker background task (24h stats refresh)

You should see: INFO excentra: Server listening port=3000

---

### 5. Try it out

**Register:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "trader@example.com", "password": "password123"}'
```

**Login:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "trader@example.com", "password": "password123"}'
```

**Deposit funds:**
```bash
curl -X POST http://localhost:3000/api/v1/balances/deposit \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"asset": "USDT", "amount": "10000"}'
```

**Place a limit order:**
```bash
curl -X POST http://localhost:3000/api/v1/orders \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"pair": "BTC/USDT", "side": "buy", "order_type": "limit", "price": "80000", "quantity": "0.1"}'
```

**View the order book:**
```bash
curl http://localhost:3000/api/v1/orderbook/BTC/USDT
```

**WebSocket (Postman or wscat):**
```bash
wscat -c ws://localhost:3000/ws
# Then send:
{"type": "subscribe", "channel": "orderbook:BTC/USDT"}
```

---

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                         Clients                               │
│              (REST via HTTP · WebSocket)                      │
└────────────────────────┬──────────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────────────┐
│                      axum HTTP Server                         │
│   ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│   │  Auth Layer │  │  REST Routes │  │  WebSocket Route  │    │
│   │ (JWT Middleware) │  /api/v1/* │  │  /ws             │    │
│   └─────────────┘  └──────┬───────┘  └────────┬─────────┘    │
└──────────────────────────┼───────────────────┼───────────────┘
                           │                   │
          ┌────────────────▼──┐     ┌──────────▼─────────────┐
          │   Service Layer   │     │    WebSocket Manager    │
          │  OrderService     │     │  (subscription routing) │
          │  TradeService     │     └──────────┬─────────────┘
          │  BalanceService   │                │
          │  AuthService      │      broadcast::Sender<WsEvent>
          └────────┬──────────┘                │
                   │                           │
     ┌─────────────▼─────────────┐             │
     │     Matching Engine       │─────────────┘
     │  Exchange (multi-pair)    │   emits trades, book updates
     │  OrderBook (per-pair)     │
     │  BTreeMap<Price, VecDeque>│
     └─────────────┬─────────────┘
                   │
     ┌─────────────▼─────────────┐
     │       PostgreSQL          │
     │  users · orders · trades  │
     │  balances · trading_pairs │
     └───────────────────────────┘
```

### Matching Engine

Each trading pair has its own in-memory `OrderBook`:

- **Bids:** `BTreeMap<Reverse<Decimal>, VecDeque<Order>>` — highest price first
- **Asks:** `BTreeMap<Decimal, VecDeque<Order>>` — lowest price first
- **Cancel index:** `HashMap<Uuid, Decimal>` — O(1) cancel lookup

The matcher walks the opposite side of the book, filling at resting order prices (FIFO within each price level). Partial fills are supported. Matched trades are persisted to PostgreSQL and broadcast over WebSocket.

### WebSocket Architecture

A `tokio::broadcast` channel carries events from the engine to all connected clients. The WebSocket manager routes events to subscribers based on their active subscriptions — only BTC/USDT subscribers receive BTC/USDT order book updates.

### Balance Holds

When a user places an order, funds move from `available` → `held`. On match, held funds transfer to the counterparty. On cancel, they return to available. This prevents double-spend without a separate ledger.

---

## Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Language | Rust | Memory safety, performance, strong type system |
| HTTP Framework | axum | Ergonomic, tower-compatible, async-first |
| Async Runtime | tokio | Industry standard for async Rust |
| Database | PostgreSQL + sqlx | Compile-time query checking, async support |
| Auth | JWT + argon2 | Stateless auth; argon2 is the gold standard for password hashing |
| Financial Math | rust_decimal | Exact decimal arithmetic |
| Serialization | serde + serde_json | Universal Rust serialization |
| Logging | tracing + tracing-subscriber | Structured, async-aware logging |
| Price Seeding | CoinGecko API | Free, real market prices, no API key required |

---

## Key Technical Decisions

### Prices as strings, not floats

All prices and quantities are `rust_decimal::Decimal` internally and serialized as JSON strings. IEEE 754 floating point cannot represent most decimal fractions exactly — using floats for financial data is a latent correctness bug.

### BTreeMap for the order book

`BTreeMap` keeps prices sorted automatically: O(log n) to find the best price, O(k) to walk k price levels during matching. A `HashMap` would give O(1) lookup but no ordering — unusable for a matching engine. Within each price level, `VecDeque` provides O(1) FIFO access for time-priority matching.

### In-memory matching, persistent everything else

Matching happens in memory for speed. Every placement, fill, and cancellation is immediately persisted to PostgreSQL. On restart, the order book rebuilds from open orders in the database — the engine is stateless, the database is the source of truth.

### System user for order book seeding

Seed orders placed at startup are backed by a real system user with pre-seeded balances. The engine code path is identical for seed and real orders — no special-casing, no divergent logic.

### MutexGuard scope and async safety

Holding a `MutexGuard` across an `.await` is a deadlock risk in async Rust. Guards are always dropped (by limiting scope or using temporaries) before any async calls. This was a non-obvious constraint in async Rust lifetime management that shaped several service-layer patterns.

---

## Roadmap

| Phase | Description | Status |
|---|---|---|
| 1 | Project setup, domain types, database schema | ✅ Complete |
| 2 | Matching engine (in-memory, FIFO) | ✅ Complete |
| 3 | Database layer (sqlx queries) | ✅ Complete |
| 4 | Auth (JWT + argon2) | ✅ Complete |
| 5 | REST API | ✅ Complete |
| 6 | WebSocket streaming | ✅ Complete |
| 7 | Ticker service + CoinGecko price seeding | ✅ Complete |
| 8 | Observability (tracing, metrics) | ✅ Complete |
| 9 | Ethereum Sepolia testnet integration | 📋 Planned |
| 10 | Frontend (React / Next.js) | 🔄 In Progress |
| 11 | Deployment (Docker, VPS, CI/CD) | 📋 Planned |

### Blockchain Integration (Phase 9)

Deposit and withdrawal on Ethereum Sepolia testnet:
- HD wallet with per-user deposit addresses (BIP-44)
- Blockchain listener service watches for incoming transactions
- Balance credited after N block confirmations
- Signed withdrawal transactions broadcast via Alchemy

---

## Future Work

- **Fee system:** Maker/taker model with fee collection
- **Stop-loss / take-profit:** Triggered order types
- **Margin trading:** Leveraged positions and a liquidation engine
- **Order book snapshots:** Faster restarts under high load
- **Rate limiting:** Per-user API limits
- **More trading pairs:** Dynamic pair addition without restart

---

## License

MIT