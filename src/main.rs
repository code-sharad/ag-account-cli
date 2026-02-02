use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

#[derive(Parser, Debug)]
#[command(name = "ag-tui")]
#[command(about = "CLI tool for displaying Antigravity account usage and quotas")]
#[command(version)]
struct Args {
    /// API URL to fetch account data from
    #[arg(short, long, default_value = "http://localhost:8040/account-limits")]
    url: String,

    /// Refresh interval in seconds (0 to disable auto-refresh)
    #[arg(short, long, default_value = "5")]
    interval: u64,

    /// Run once and exit (no auto-refresh)
    #[arg(short, long)]
    once: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ModelRateLimit {
    #[serde(rename = "isRateLimited")]
    is_rate_limited: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct Account {
    email: String,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(rename = "limits")]
    limits: Option<HashMap<String, ModelQuota>>,
    #[serde(rename = "modelRateLimits")]
    model_rate_limits: Option<HashMap<String, ModelRateLimit>>,
    #[serde(rename = "isInvalid")]
    is_invalid: Option<bool>,
    #[serde(rename = "lastUsed")]
    last_used: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModelQuota {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: f64,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiResponseWrapper {
    result: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiResponse {
    timestamp: Option<String>,
    accounts: Vec<Account>,
    models: Vec<String>,
}

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
}

fn get_short_email(email: &str) -> &str {
    email.split('@').next().unwrap_or(email)
}

fn format_timestamp(ts: u64) -> String {
    DateTime::from_timestamp_millis(ts as i64)
        .map(|d| d.with_timezone(&Local).format("%-m/%-d/%Y, %-I:%M:%S %p").to_string())
        .unwrap_or_else(|| "never".to_string())
}

fn format_reset_time(reset_time: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(reset_time) {
        let duration = dt.signed_duration_since(Utc::now());
        if duration.num_seconds() <= 0 {
            return "now".to_string();
        }
        let h = duration.num_hours();
        let m = duration.num_minutes() % 60;
        let s = duration.num_seconds() % 60;
        if h > 0 {
            format!("{}h{}m{}s", h, m, s)
        } else if m > 0 {
            format!("{}m{}s", m, s)
        } else {
            format!("{}s", s)
        }
    } else {
        reset_time.to_string()
    }
}

fn get_account_status(account: &Account) -> (&'static str, &'static str) {
    if account.is_invalid.unwrap_or(false) {
        return ("invalid", RED);
    }
    if !account.enabled.unwrap_or(true) {
        return ("disabled", DIM);
    }
    if let Some(ref rate_limits) = account.model_rate_limits {
        let limited = rate_limits.values().filter(|r| r.is_rate_limited).count();
        if limited > 0 {
            return ("limited", YELLOW);
        }
    }
    ("ok", GREEN)
}

fn count_stats(accounts: &[Account]) -> (usize, usize, usize) {
    let mut available = 0;
    let mut rate_limited = 0;
    let mut invalid = 0;

    for a in accounts {
        if a.is_invalid.unwrap_or(false) {
            invalid += 1;
        } else if a.model_rate_limits.as_ref().map(|r| r.values().any(|l| l.is_rate_limited)).unwrap_or(false) {
            rate_limited += 1;
        } else if a.enabled.unwrap_or(true) {
            available += 1;
        }
    }
    (available, rate_limited, invalid)
}

async fn fetch_data(url: &str) -> Result<ApiResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to connect to server")?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(anyhow::anyhow!("Server returned error {}", status));
    }

    let text = response.text().await.context("Failed to read response")?;

    // Try wrapped response first
    if let Ok(wrapper) = serde_json::from_str::<ApiResponseWrapper>(&text) {
        serde_json::from_str(&wrapper.result).context("Failed to parse inner JSON")
    } else {
        serde_json::from_str(&text).context("Failed to parse JSON")
    }
}

fn print_table(data: &ApiResponse) {
    let timestamp = data.timestamp.clone()
        .unwrap_or_else(|| Local::now().format("%-m/%-d/%Y, %-I:%M:%S %p").to_string());

    let (available, rate_limited, invalid) = count_stats(&data.accounts);
    let total = data.accounts.len();

    // Header
    println!("{}{}Account Limits{} {}({}){}", BOLD, CYAN, RESET, DIM, timestamp, RESET);
    println!(
        "Accounts: {} total, {}{} available{}, {}{} rate-limited{}, {}{} invalid{}",
        total,
        GREEN, available, RESET,
        YELLOW, rate_limited, RESET,
        RED, invalid, RESET
    );
    println!();

    // Account summary table
    println!(
        "{}{:<20} {:<15} {:<25} {:<25}{}",
        BOLD, "Account", "Status", "Last Used", "Quota Reset", RESET
    );
    println!("{}", "-".repeat(85));

    for account in &data.accounts {
        let email = get_short_email(&account.email);
        let (status, color) = get_account_status(account);

        let status_display = if status == "limited" {
            if let Some(ref rl) = account.model_rate_limits {
                let limited = rl.values().filter(|r| r.is_rate_limited).count();
                format!("({}/{}) limited", limited, rl.len())
            } else {
                status.to_string()
            }
        } else {
            status.to_string()
        };

        let last_used = account.last_used
            .map(format_timestamp)
            .unwrap_or_else(|| "never".to_string());

        let reset = account.limits.as_ref()
            .and_then(|limits| {
                limits.values()
                    .filter_map(|q| q.reset_time.as_ref())
                    .min()
                    .map(|t| {
                        DateTime::parse_from_rfc3339(t)
                            .map(|d| format_timestamp(d.timestamp_millis() as u64))
                            .unwrap_or_else(|_| "N/A".to_string())
                    })
            })
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{:<20} {}{:<15}{} {:<25} {:<25}",
            email, color, status_display, RESET, last_used, reset
        );
    }

    println!();

    // Model quota table
    // Build header
    print!("{}{:<28}", BOLD, "Model");
    for account in &data.accounts {
        print!("{:<20}", get_short_email(&account.email));
    }
    println!("{}", RESET);
    println!("{}", "-".repeat(28 + data.accounts.len() * 20));

    // Model rows
    for model in &data.models {
        print!("{:<28}", model);

        for account in &data.accounts {
            let cell = if let Some(ref limits) = account.limits {
                if let Some(quota) = limits.get(model) {
                    let pct = (quota.remaining_fraction * 100.0) as u32;
                    let is_limited = account.model_rate_limits.as_ref()
                        .and_then(|r| r.get(model))
                        .map(|l| l.is_rate_limited)
                        .unwrap_or(false);

                    if quota.remaining_fraction <= 0.0 || is_limited {
                        let wait = quota.reset_time.as_ref()
                            .map(|t| format!("{}% (wait {})", pct, format_reset_time(t)))
                            .unwrap_or_else(|| format!("{}%", pct));
                        format!("{}{:<20}{}", RED, wait, RESET)
                    } else if quota.remaining_fraction < 0.3 {
                        format!("{}{:<20}{}", YELLOW, format!("{}%", pct), RESET)
                    } else {
                        format!("{}{:<20}{}", GREEN, format!("{}%", pct), RESET)
                    }
                } else {
                    format!("{}{:<20}{}", DIM, "N/A", RESET)
                }
            } else {
                format!("{}{:<20}{}", DIM, "N/A", RESET)
            };
            print!("{}", cell);
        }
        println!();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    loop {
        clear_screen();

        match fetch_data(&args.url).await {
            Ok(data) => print_table(&data),
            Err(e) => {
                println!("{}Error: {}{}", RED, e, RESET);
                println!("\nMake sure the proxy is running at {}", args.url);
            }
        }

        if args.once || args.interval == 0 {
            break;
        }

        println!("\n{}Refreshing every {}s... (Ctrl+C to exit){}", DIM, args.interval, RESET);
        tokio::time::sleep(Duration::from_secs(args.interval)).await;
    }

    Ok(())
}
