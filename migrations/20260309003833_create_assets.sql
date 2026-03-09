-- Add migration script here
CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(50) NOT NULL,
    decimals SMALLINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO assets (symbol, name, decimals) VALUES
('BTC', 'Bitcoin', 8),
('ETH', 'Ethereum', 18),
('SOL', 'Solana', 9),
('USDT', 'Tether', 6);