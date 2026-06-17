# NetWatch

A lightweight, SQLite-backed network traffic monitor for Linux systems with a stunning Terminal User Interface (TUI) and Command Line Interface (CLI).

## Features

- **Daemon-Client Architecture**: `netwatchd` runs in the background collecting stats; `netwatch` displays them without holding locks or overhead.
- **SQLite Storage**: Uses Write-Ahead Logging (WAL) and synchronous level `NORMAL` for lightweight, reliable, concurrency-friendly storage.
- **Tiered Aggregation**: Historical data is downsampled automatically to minute, hourly, and daily aggregates to save disk space.
- **Diagnostics**: `netwatch doctor` validates system config, database integrity, and interface collection.
- **Packaging**: Ready-to-go `systemd` units, man pages, Arch `PKGBUILD`, and Debian/RedHat configuration templates.

## Installation

### 1. Build from Source
```bash
cargo build --release
```

### 2. Enable systemd Daemon Service

You can run the background daemon as a **User-level service** (starts when you log in) or a **System-wide service** (starts automatically at boot).

#### Option A: User-level Service (Default)
```bash
mkdir -p ~/.config/systemd/user/
cp assets/systemd/netwatchd.service ~/.config/systemd/user/netwatchd.service
systemctl --user daemon-reload
systemctl --user enable --now netwatchd.service
```

#### Option B: System-wide Service
```bash
sudo cp target/release/netwatchd /usr/local/bin/
sudo cp assets/systemd/netwatchd-system.service /etc/systemd/system/netwatchd.service
# Edit service to set your actual user:
# sudo nano /etc/systemd/system/netwatchd.service
sudo systemctl daemon-reload
sudo systemctl enable --now netwatchd.service
```

## Configuration

