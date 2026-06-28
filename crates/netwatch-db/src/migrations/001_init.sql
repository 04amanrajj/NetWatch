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

CREATE TABLE IF NOT EXISTS samples_minute (
    ts INTEGER NOT NULL,
    interface_id INTEGER NOT NULL,
    rx_bytes INTEGER NOT NULL,
    tx_bytes INTEGER NOT NULL,
    rx_rate_avg INTEGER NOT NULL DEFAULT 0,
    tx_rate_avg INTEGER NOT NULL DEFAULT 0,
    rx_rate_max INTEGER NOT NULL DEFAULT 0,
    tx_rate_max INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (ts, interface_id),
    FOREIGN KEY (interface_id) REFERENCES interfaces(id)
);

CREATE TABLE IF NOT EXISTS samples_hourly (
    ts INTEGER NOT NULL,
    interface_id INTEGER NOT NULL,
    rx_bytes INTEGER NOT NULL,
    tx_bytes INTEGER NOT NULL,
    rx_rate_avg INTEGER NOT NULL DEFAULT 0,
    tx_rate_avg INTEGER NOT NULL DEFAULT 0,
    rx_rate_max INTEGER NOT NULL DEFAULT 0,
    tx_rate_max INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (ts, interface_id),
    FOREIGN KEY (interface_id) REFERENCES interfaces(id)
);

CREATE TABLE IF NOT EXISTS samples_daily (
    ts INTEGER NOT NULL,
    interface_id INTEGER NOT NULL,
    rx_bytes INTEGER NOT NULL,
    tx_bytes INTEGER NOT NULL,
    rx_rate_avg INTEGER NOT NULL DEFAULT 0,
    tx_rate_avg INTEGER NOT NULL DEFAULT 0,
    rx_rate_max INTEGER NOT NULL DEFAULT 0,
    tx_rate_max INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (ts, interface_id),
    FOREIGN KEY (interface_id) REFERENCES interfaces(id)
);

CREATE TABLE IF NOT EXISTS daemon_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,
    kind TEXT NOT NULL,
    message TEXT NOT NULL,
    acknowledged INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_samples_raw_iface_ts ON samples_raw (interface_id, ts);
CREATE INDEX IF NOT EXISTS idx_samples_raw_ts ON samples_raw (ts);
CREATE INDEX IF NOT EXISTS idx_samples_minute_iface_ts ON samples_minute (interface_id, ts);
CREATE INDEX IF NOT EXISTS idx_samples_minute_ts ON samples_minute (ts);
CREATE INDEX IF NOT EXISTS idx_samples_hourly_iface_ts ON samples_hourly (interface_id, ts);
CREATE INDEX IF NOT EXISTS idx_samples_hourly_ts ON samples_hourly (ts);
CREATE INDEX IF NOT EXISTS idx_samples_daily_iface_ts ON samples_daily (interface_id, ts);
CREATE INDEX IF NOT EXISTS idx_samples_daily_ts ON samples_daily (ts);
