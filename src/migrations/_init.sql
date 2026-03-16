CREATE TABLE IF NOT EXISTS stylus_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    tx_hash TEXT NOT NULL UNIQUE,
    contract_address TEXT NOT NULL,
    eth_value TEXT,
    is_whale,
	is_stylus BOOLEAN,
    detected_at DATETIME DEFAULT CURRENT_TIMESTAMP
);