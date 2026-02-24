-- Pay-as-you-go billing records.
-- Each row represents one continuous active window for an instance.
-- `ended_at` is NULL while the instance is still active; it is set when the
-- instance is deactivated and `amount_cents` is computed at that point.
CREATE TABLE billing_records (
    id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    instance_id INTEGER NOT NULL REFERENCES instances(id),
    user_id     INTEGER NOT NULL REFERENCES users(id),
    started_at  BIGINT  NOT NULL,
    ended_at    BIGINT,
    amount_cents INTEGER NOT NULL DEFAULT 0
);
