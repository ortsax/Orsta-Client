-- SQLite ≥ 3.35.0 supports DROP COLUMN; earlier versions do not.
ALTER TABLE users DROP COLUMN created_at;
