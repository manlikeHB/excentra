-- Add migration script here

ALTER TABLE assets ADD COLUMN coingecko_id VARCHAR;

UPDATE assets SET coingecko_id = 'bitcoin' WHERE symbol = 'BTC';
UPDATE assets SET coingecko_id = 'ethereum' WHERE symbol = 'ETH';
UPDATE assets SET coingecko_id = 'solana' WHERE symbol = 'SOL';
UPDATE assets SET coingecko_id = 'tether' WHERE symbol = 'USDT';