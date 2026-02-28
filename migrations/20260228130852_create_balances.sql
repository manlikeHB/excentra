-- Add migration script here
CREATE TABLE balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    asset VARCHAR(10) NOT NULL,       -- "BTC", "USDT", etc.
    available DECIMAL NOT NULL DEFAULT 0, -- free to use
    held DECIMAL NOT NULL DEFAULT 0,      -- locked in open orders
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, asset)            -- one balance row per user per asset
);