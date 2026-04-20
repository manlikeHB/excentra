-- Add migration script here
Alter TABLE trades ADD COLUMN taker_side order_side NOT NULL;