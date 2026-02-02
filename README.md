# Antigravity Account TUI

A Terminal User Interface (TUI) for monitoring Antigravity Claude Proxy account usage and quotas in real-time.

## Features

- **Real-time account monitoring** - View all accounts with status, last used time, and quota reset times
- **Model quota table** - See per-model quota percentages for each account in a table format
- **Auto-refresh** - Data refreshes automatically every 30 seconds
- **Keyboard navigation** - Full keyboard control with vim-style bindings
- **Color-coded status** - Green for available, yellow for rate-limited, red for exhausted/invalid
- **Wait time display** - Shows remaining wait time for rate-limited quotas

## Installation

### Prerequisites

- Rust 1.70+ installed
- The Antigravity Claude Proxy server running on `localhost:8040`

### Build

```bash
cd ag-account-tui
cargo build --release
```

The binary will be at `./target/release/ag-tui.exe` (Windows) or `./target/release/ag-tui` (Linux/macOS).

## Usage

### Run

```bash
# Direct run with cargo
cargo run --release

# Run with custom URL
cargo run --release -- --url http://localhost:8080/account-limits

# Run with debug mode to see raw responses
cargo run --release -- --debug

# Or run the binary directly
./target/release/ag-tui

# With custom URL
./target/release/ag-tui --url http://localhost:8080/account-limits
```

### Command Line Options

```
ag-tui [OPTIONS]

Options:
  -u, --url <URL>    API URL to fetch account data from [default: http://localhost:8040/account-limits]
  -d, --debug        Enable debug mode to show raw responses
  -h, --help         Print help
  -V, --version      Print version
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up in models list |
| `↓` / `j` | Scroll down in models list |
| `PgUp` | Page up (10 items) |
| `PgDn` | Page down (10 items) |
| `Home` | Go to top |
| `End` | Go to bottom |
| `r` | Refresh data manually |
| `a` | Toggle auto-refresh |
| `h` or `?` | Show help popup |
| `q` or `Esc` | Quit |
| `Ctrl+C` | Force quit |

### Layout

The TUI displays:

1. **Header** - Shows timestamp, account summary (total, available, rate-limited, invalid)

2. **Accounts Table** - Shows all accounts with:
   - Account email (shortened)
   - Status (ok, limited with count, invalid, disabled)
   - Last used timestamp
   - Quota reset time

3. **Models Table** - Shows model quotas with:
   - Model name in first column
   - Quota percentage for each account
   - Color-coded: Green (>30%), Yellow (10-30%), Red (<10%)
   - Wait time shown for rate-limited quotas (e.g., "0% (wait 1h23m45s)")

4. **Footer** - Keyboard shortcuts reference

### API Endpoint

The TUI fetches data from:
```
http://localhost:8040/account-limits
```

Make sure your Antigravity Claude Proxy server is running before starting the TUI.

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal handling
- `reqwest` - HTTP client for API calls
- `tokio` - Async runtime
- `serde` - JSON serialization
- `chrono` - Date/time handling
- `clap` - Command line argument parsing
- `anyhow` - Error handling

## Troubleshooting

### "Failed to parse JSON" Error

If you see an error about JSON parsing:

1. **Check if the proxy is running**: Make sure the Antigravity Claude Proxy is running on the expected port (default: 8040)
2. **Use debug mode**: Run with `--debug` flag to see the raw response
3. **Check the URL**: Verify the URL is correct with `--url`
4. **Test with curl**: Try manually fetching data:
   ```bash
   curl http://localhost:8040/account-limits
   ```

### Connection Errors

If you see "Failed to connect to server":

- Verify the proxy is running: `npm start` in the proxy directory
- Check if the port is correct (default 8040)
- Make sure no firewall is blocking the connection

### HTML Instead of JSON

If the error mentions "Server returned HTML instead of JSON", the endpoint might be returning an error page. Check that:
- The proxy server is fully initialized
- The `/account-limits` endpoint exists in your proxy version

## License

MIT
