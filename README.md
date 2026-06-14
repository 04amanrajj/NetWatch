# NetWatch

A lightweight, SQLite-backed network traffic monitor for Linux systems with a stunning Terminal User Interface (TUI) and Command Line Interface (CLI).

## Features

- **Daemon-Client Architecture**: `netwatchd` runs in the background collecting stats; `netwatch` displays them without holding locks or overhead.
- **SQLite Storage**: Uses Write-Ahead Logging (WAL) and synchronous level `NORMAL` for lightweight, reliable, concurrency-friendly storage.
- **Tiered Aggregation**: Historical data is downsampled automatically to minute, hourly, and daily aggregates to save disk space.
- **Diagnostics**: `netwatch doctor` validates system config, database integrity, and interface collection.
- **Packaging**: Ready-to-go `systemd` units, man pages, Arch `PKGBUILD`, and Debian/RedHat configuration templates.

## Installation

