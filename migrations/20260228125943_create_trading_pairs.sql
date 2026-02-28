-- Add migration script here
CREATE TABLE trading_pairs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    base_asset VARCHAR(10) NOT NULL,    -- "BTC", "ETH", "SOL"
    quote_asset VARCHAR(10) NOT NULL,   -- "USDT"
    symbol VARCHAR(21) NOT NULL UNIQUE, -- "BTC/USDT"
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);