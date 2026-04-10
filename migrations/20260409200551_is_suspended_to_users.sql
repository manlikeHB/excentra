-- Add migration script here
ALTER TABLE users
ADD COLUMN is_suspended BOOL NOT NULL DEFAULT FALSE;