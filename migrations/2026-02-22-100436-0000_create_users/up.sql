CREATE TABLE IF NOT EXISTS users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    passkey TEXT,
    eakey TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS user_property (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    instance_status TEXT NOT NULL DEFAULT 'inactive',
    instance_usage REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS instances (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    instances_count INTEGER NOT NULL DEFAULT 0,
    expected_consumption REAL NOT NULL DEFAULT 0.0,
    instances_overall_consumption REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS billing (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    amount_in_wallet REAL NOT NULL DEFAULT 0.0,
    amount_spent REAL NOT NULL DEFAULT 0.0,
    total_amount_spent REAL NOT NULL DEFAULT 0.0,
    average_hourly_consumption REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);