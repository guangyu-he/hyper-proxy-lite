# hyper-proxy-lite

A lightweight HTTP/HTTPS forward proxy server written in Rust with domain filtering support.

## Features

- **HTTP Proxy** - Forward HTTP requests to destination servers
- **HTTPS Tunneling** - Handle CONNECT requests for secure HTTPS traffic
- **Domain Filtering** - Block or allow domains using blacklist/whitelist modes
- **Async Architecture** - Non-blocking I/O powered by Tokio for high concurrency
- **Flexible Configuration** - Configure via TOML file or command-line arguments

## Installation

```bash
git clone https://github.com/guangyu-he/hyper-proxy-lite.git
cd hyper-proxy-lite
cargo build --release
```

## Usage

### Basic Usage (No Filtering)

```bash
cargo run --release
```

The proxy server starts on `127.0.0.1:8080`.

### With Blacklist (Block Specific Domains)

```bash
cargo run --release -- --blacklist "example.com,blocked.org"
```

### With Whitelist (Allow Only Specific Domains)

```bash
cargo run --release -- --whitelist "allowed.com,trusted.org"
```

### With Configuration File

```bash
cargo run --release -- --filter filter_rules.toml
```

## Configuration

### TOML Configuration File

Create a TOML file with filter rules:

```toml
mode = "Blacklist"  # or "Whitelist"
domains = [
    "example.com",
    "blocked.org"
]
```

### Command-Line Arguments

| Argument                | Description                              |
|-------------------------|------------------------------------------|
| `--filter <path>`       | Load filter rules from a TOML file       |
| `--blacklist <domains>` | Comma-separated list of domains to block |
| `--whitelist <domains>` | Comma-separated list of domains to allow |

## How It Works

1. The proxy listens for incoming connections on `127.0.0.1:8080`
2. For each request, it extracts the target domain
3. The domain is checked against filter rules (if configured)
4. Blocked requests receive a `403 Forbidden` response
5. Allowed requests are:
    - **HTTP**: Forwarded to the destination server
    - **HTTPS (CONNECT)**: Tunneled via bidirectional relay

## Dependencies

- [tokio](https://crates.io/crates/tokio) - Async runtime
- [hyper](https://crates.io/crates/hyper) - HTTP implementation
- [clap](https://crates.io/crates/clap) - CLI argument parsing
- [serde](https://crates.io/crates/serde) + [toml](https://crates.io/crates/toml) - Configuration parsing
- [anyhow](https://crates.io/crates/anyhow) - Error handling

## License

MIT
