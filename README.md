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
| 8 | Observability (tracing, metrics) | 🔄 In Progress |
| 9 | Ethereum Sepolia testnet integration | 📋 Planned |
| 10 | Frontend (React / Next.js) | 📋 Planned |
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