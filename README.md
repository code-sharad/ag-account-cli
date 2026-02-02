# ag-quota

A CLI tool for monitoring Antigravity Claude Proxy account usage and quotas in real-time.

## Installation

### From crates.io

```bash
cargo install ag-quota
```

### From source

```bash
git clone https://github.com/code-sharad/ag-quota.git
cd ag-quota
cargo build --release
```

## Features

- **Real-time monitoring** - Auto-refreshes every 5 seconds (configurable)
- **Color-coded status** - Green for available, yellow for rate-limited, red for exhausted/invalid
- **Wait time display** - Shows remaining wait time for rate-limited quotas
- **Simple CLI** - No TUI dependencies, just prints colored tables

## Usage

```bash
# Run with default settings (localhost:8040, 5s refresh)
ag-quota

# Custom API URL
ag-quota --url http://localhost:8080/account-limits

# Custom refresh interval (10 seconds)
ag-quota --interval 10

# Run once and exit (no auto-refresh)
ag-quota --once
```

### Command Line Options

```
ag-quota [OPTIONS]

Options:
  -u, --url <URL>        API URL [default: http://localhost:8040/account-limits]
  -i, --interval <SECS>  Refresh interval in seconds [default: 5]
  -o, --once             Run once and exit
  -h, --help             Print help
  -V, --version          Print version
```

## Output

The CLI displays:

1. **Header** - Timestamp and account summary (total, available, rate-limited, invalid)

2. **Accounts Table**
   - Account email
   - Status (ok, limited, invalid, disabled)
   - Last used timestamp
   - Quota reset time

3. **Models Table**
   - Model name
   - Quota percentage per account
   - Color-coded: Green (>30%), Yellow (10-30%), Red (<10%)
   - Wait time for rate-limited quotas (e.g., "0% (wait 1h23m45s)")

## Requirements

- Rust 1.70+ (for building from source)
- Antigravity Claude Proxy running on the specified URL

## License

MIT
