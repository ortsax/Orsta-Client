-- Add created_at to users to support 2-month new-user promotion tracking.
ALTER TABLE users ADD COLUMN created_at BIGINT NOT NULL DEFAULT (strftime('%s', 'now'));
