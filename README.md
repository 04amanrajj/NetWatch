# NetWatch

A lightweight, SQLite-backed network traffic monitor for Linux systems with a stunning Terminal User Interface (TUI) and Command Line Interface (CLI).

## Features

- **Daemon-Client Architecture**: `netwatchd` runs in the background collecting stats; `netwatch` displays them without holding locks or overhead.
- **SQLite Storage**: Uses Write-Ahead Logging (WAL) and synchronous level `NORMAL` for lightweight, reliable, concurrency-friendly storage.
- **Tiered Aggregation**: Historical data is downsampled automatically to minute, hourly, and daily aggregates to save disk space.
- **Diagnostics**: `netwatch doctor` validates system config, database integrity, and interface collection.
- **Packaging**: Ready-to-go `systemd` units, man pages, Arch `PKGBUILD`, and Debian/RedHat configuration templates.

## Quick Start & Testing (e.g., in Docker Ubuntu)

If you are testing NetWatch in a fresh environment, such as a Docker Ubuntu container, follow these steps:

### 1. Install System Dependencies
Install the required build tools and SQLite development libraries.

* **Ubuntu / Debian**:
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    sqlite3 \
    libsqlite3-dev
  ```
* **Arch Linux**:
  ```bash
  sudo pacman -Syu --needed base-devel sqlite
  ```

### 2. Install Rust & Cargo
If you don't have Rust/Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 3. Clone and Build the Project
```bash
git clone https://github.com/04amanrajj/NetWatch.git
cd NetWatch
cargo build --release
```

### 4. Test End-to-End
To verify that the daemon and TUI are working correctly:
```bash
# Run the daemon in the background
./target/release/netwatchd &

# Generate some network traffic to collect
apt-get update # or curl a few webpages

# Launch the interactive terminal UI
./target/release/netwatch
```
*(Press `q` to exit the TUI.)*

---

## Production Service Setup

For standard Linux host deployments, you can run the background daemon as a user-level service or a system-wide service using `systemd`.

### Option 1: User-level Service (Starts when you log in)

1. **Install the binary** to your local cargo bin:
   ```bash
   mkdir -p ~/.cargo/bin
   cp target/release/netwatchd ~/.cargo/bin/
   ```

2. **Copy the service file**:
   ```bash
   mkdir -p ~/.config/systemd/user/
   cp assets/systemd/netwatchd.service ~/.config/systemd/user/netwatchd.service
   ```

3. **Reload systemd, enable, and start the service**:
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable --now netwatchd.service
   ```

4. **Verify it is running**:
   ```bash
   systemctl --user status netwatchd.service
   ```

### Option 2: System-wide Service (Starts automatically at boot)

1. **Install the binary** to `/usr/local/bin`:
   ```bash
   sudo cp target/release/netwatchd /usr/local/bin/
   ```

2. **Copy the service file**:
   ```bash
   sudo cp assets/systemd/netwatchd-system.service /etc/systemd/system/netwatchd.service
   ```

3. **Configure the runner user**:
   Open `/etc/systemd/system/netwatchd.service` and replace `YOUR_USERNAME_HERE` with the system user you want to run the daemon (e.g., your username or `root`):
   ```bash
   sudo nano /etc/systemd/system/netwatchd.service
   ```

4. **Reload systemd, enable, and start the service**:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now netwatchd.service
   ```

5. **Verify it is running**:
   ```bash
   systemctl status netwatchd.service
   ```

---

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

### Options Breakdown

* **`sample_interval`** (default: `1`): The interval (in seconds) at which the background daemon `netwatchd` queries system interfaces. A lower value provides more real-time/granular stats but consumes slightly more CPU.
* **`database`** (default: `"~/.local/share/netwatch/netwatch.db"`): The file path where the SQLite database is located. Tilde (`~`) is automatically expanded to the user's home directory.
* **`ignore`** (default: `["docker*", "virbr*", "veth*"]`): A list of glob patterns for interface names that the daemon should skip collecting stats for. This prevents virtual bridge or container adapters from cluttering your logs.
* **`theme`** (default: `"default"`): The color/styling theme applied when launching the TUI monitor (`netwatch`).
* **`units`** (default: `"auto"`): The format used to display traffic speeds and data sizes. Options include:
  * `"auto"`: Formats automatically to the most appropriate human-readable size (e.g. KB, MB, kbps, Mbps).
  * `"bytes"`: Forces output in Bytes/second and Megabytes.
  * `"bits"`: Forces output in Bits/second (typical ISP speed format).
* **`history_days`** (default: `365`): The data retention period (in days). Raw database samples older than this threshold are pruned to prevent the database from growing indefinitely.
* **`batch_write_interval`** (default: `5`): The interval (in seconds) at which collected samples are written/committed to the SQLite database in batches. This minimizes disk write operations and lock contention.
* **`skip_loopback`** (default: `true`): A boolean config option (can be set to `true`/`false`) specifying whether to ignore the loopback (`lo`) local network interface.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
