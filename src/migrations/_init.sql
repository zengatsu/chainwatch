CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    tx_hash TEXT NOT NULL UNIQUE,
    from_addr TEXT NOT NULL,
    to_addr TEXT NOT NULL,
    eth_value TEXT,
    is_whale BOOLEAN,
	is_stylus BOOLEAN,
    detected_at DATETIME DEFAULT CURRENT_TIMESTAMP
);