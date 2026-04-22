-- Add migration script here
ALTER TABLE refresh_tokens ADD COLUMN used_at TIMESTAMPTZ;