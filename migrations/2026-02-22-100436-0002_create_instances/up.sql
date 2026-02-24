-- WhatsApp bot instances managed by Orsta.
-- `active` is stored as 0/1; billing only accrues while active = 1.
-- `country_code` must be one of the 180 WhatsApp-supported country codes (ISO 3166-1 alpha-2).
CREATE TABLE instances (
    id           INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL REFERENCES users(id),
    country_code TEXT    NOT NULL,
    phone_number TEXT    NOT NULL,
    active       INTEGER NOT NULL DEFAULT 0,
    created_at   BIGINT  NOT NULL DEFAULT (strftime('%s', 'now'))
);
