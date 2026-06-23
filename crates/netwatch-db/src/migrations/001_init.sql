-- NetWatch initial schema

CREATE TABLE IF NOT EXISTS interfaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    mac TEXT,
    first_seen INTEGER NOT NULL,
    last_seen INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS samples_raw (
    ts INTEGER NOT NULL,
    interface_id INTEGER NOT NULL,
    rx_bytes INTEGER NOT NULL,
    tx_bytes INTEGER NOT NULL,
    rx_rate INTEGER,
    tx_rate INTEGER,
    PRIMARY KEY (ts, interface_id),
    FOREIGN KEY (interface_id) REFERENCES interfaces(id)
);
