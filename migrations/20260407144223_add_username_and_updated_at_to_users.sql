-- Add migration script here
ALTER TABLE users
ADD COLUMN username VARCHAR(50) UNIQUE,
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();