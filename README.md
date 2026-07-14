# NetWatch

A lightweight, SQLite-backed network traffic monitor for Linux systems with a stunning Terminal User Interface (TUI) and Command Line Interface (CLI).

## Features

- **Daemon-Client Architecture**: `netwatchd` runs in the background collecting stats; `netwatch` displays them without holding locks or overhead.
- **SQLite Storage**: Uses Write-Ahead Logging (WAL) and synchronous level `NORMAL` for lightweight, reliable, concurrency-friendly storage.
- **Tiered Aggregation**: Historical data is downsampled automatically to minute, hourly, and daily aggregates to save disk space.
- **Diagnostics**: `netwatch doctor` validates system config, database integrity, and interface collection.
- **Packaging**: Ready-to-go `systemd` units, man pages, Arch `PKGBUILD`, and Debian/RedHat configuration templates.

## Installation & Setup

### 1. Install System Dependencies
Ensure you have the required compiler and SQLite development libraries installed:

**On Ubuntu / Debian**:
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config sqlite3 libsqlite3-dev curl
```

### 2. Install Rust and Cargo
If you don't have Rust installed, install it using `rustup`:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 3. Build the Project
Compile the binaries in release mode:
```bash
cargo build --release
```
This builds two binaries:
- `target/release/netwatchd`: The background statistics daemon.
- `target/release/netwatch`: The terminal user interface (TUI) client.

### 4. Enable Background Daemon (System-wide Service)
Configure `netwatchd` to run automatically at boot as a system-wide `systemd` service:

```bash
# Install the compiled daemon binary
sudo cp target/release/netwatchd /usr/local/bin/

# Copy the service configuration
sudo cp assets/systemd/netwatchd-system.service /etc/systemd/system/netwatchd.service

# Edit /etc/systemd/system/netwatchd.service to replace `YOUR_USERNAME_HERE` with your system username
sudo nano /etc/systemd/system/netwatchd.service

# Reload systemd and start/enable the service
sudo systemctl daemon-reload
sudo systemctl enable --now netwatchd
```

### 5. Launch the Client (TUI)
Once the service is active, run the compiled client to monitor network traffic in real-time:
```bash
./target/release/netwatch
```
*To exit the TUI interface, press `q`.*

### Docker Testing (Alternative)
If you are testing this inside a standard Docker container (where systemd is not booted by default), you can run the daemon in the background using `nohup` instead:
```bash
# Start daemon in the background
nohup ./target/release/netwatchd > /var/log/netwatchd.log 2>&1 &

# Run TUI client
./target/release/netwatch
```

## Configuration

A configuration file is placed at `~/.config/netwatch/config.toml`:

```toml
sample_interval = 1
database = "~/.local/share/netwatch/netwatch.db"
ignore = ["docker*", "virbr*", "veth*"]
theme = "default"
units = "auto"
history_days = 365
batch_write_interval = 5
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
