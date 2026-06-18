//! Cryptocurrency prices via the CoinGecko public API.
//!
//! Fetched by the daemon but rate-limited by `@huma-kripto-ttl`: the price is
//! cached in `$XDG_RUNTIME_DIR/huma-kripto.cache` and only re-fetched once the
//! TTL elapses, so the daemon tick (seconds) never hammers the API. A failed
//! fetch falls back to the last good value instead of blanking the widget.

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::tmux;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn cache_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(dir).join("huma-kripto.cache")
}

/// Group an integer with thousands separators: `43210` → `43,210`.
pub fn group_thousands(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Pull `coin`'s `currency` price out of a CoinGecko `simple/price` response,
/// e.g. `{"bitcoin":{"usd":43210.5}}` → `Some(43210.5)`. No JSON dependency:
/// we locate the coin key, then its currency key, then read the number.
pub fn parse_price(json: &str, coin: &str, currency: &str) -> Option<f64> {
    let coin_key = format!("\"{coin}\"");
    let after_coin = &json[json.find(&coin_key)? + coin_key.len()..];
    let cur_key = format!("\"{currency}\"");
    let after_cur = &after_coin[after_coin.find(&cur_key)? + cur_key.len()..];
    let num: String = after_cur
        .trim_start_matches([':', ' '])
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num.parse().ok()
}

fn fmt_price(price: f64, symbol: &str, coin: &str, multi: bool) -> String {
    let n = group_thousands(price.round() as u64);
    if multi {
        format!("{coin} {symbol}{n}")
    } else {
        format!("{symbol}{n}")
    }
}

/// Format the whole widget from a CoinGecko response for the configured coins.
pub fn format_widget(json: &str, cfg: &Config) -> String {
    let multi = cfg.kripto_coins.len() > 1;
    cfg.kripto_coins
        .iter()
        .filter_map(|coin| {
            parse_price(json, coin, &cfg.kripto_currency)
                .map(|p| fmt_price(p, &cfg.kripto_symbol, coin, multi))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn fetch(cfg: &Config) -> Option<String> {
    let ids = cfg.kripto_coins.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={ids}&vs_currencies={}",
        cfg.kripto_currency
    );
    let out = Command::new("curl")
        .args(["-s", "--max-time", "4", &url])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let json = String::from_utf8_lossy(&out.stdout);
    let s = format_widget(&json, cfg);
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn cached_value() -> Option<String> {
    let s = std::fs::read_to_string(cache_path()).ok()?;
    s.split_once('\n').map(|(_, v)| v.to_string())
}

/// Return the kripto widget, fetching only when the cache is older than the TTL.
pub fn value(cfg: &Config) -> String {
    if cfg.kripto_coins.is_empty() {
        return String::new();
    }
    if let Ok(s) = std::fs::read_to_string(cache_path()) {
        if let Some((ts, val)) = s.split_once('\n') {
            if let Ok(ts) = ts.parse::<u64>() {
                if now().saturating_sub(ts) < cfg.kripto_ttl {
                    return val.to_string();
                }
            }
        }
    }
    match fetch(cfg) {
        Some(v) => {
            let _ = std::fs::write(cache_path(), format!("{}\n{}", now(), v));
            v
        }
        None => cached_value().unwrap_or_default(),
    }
}

pub fn update(cfg: &Config) {
    tmux::set_global_option("@huma_kripto", &value(cfg));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(coins: &[&str]) -> Config {
        Config {
            interval_secs: 5,
            online_host: "x".into(),
            online_timeout_secs: 1,
            online_latency: true,
            online_up: "✓".into(),
            online_down: "✗".into(),
            battery_low: 20,
            load_mem: false,
            load_icon: "▟".into(),
            mode_prefix: "P".into(),
            mode_copy: "C".into(),
            mode_sync: "S".into(),
            mode_mouse: "M".into(),
            ssh_user_at_host: false,
            kripto_coins: coins.iter().map(|s| s.to_string()).collect(),
            kripto_currency: "usd".into(),
            kripto_symbol: "$".into(),
            kripto_ttl: 300,
            player_format: "{{artist}} - {{title}}".into(),
            player_max: 40,
            player_playing: "▶".into(),
            player_paused: "⏸".into(),
        }
    }

    #[test]
    fn thousands() {
        assert_eq!(group_thousands(100), "100");
        assert_eq!(group_thousands(43210), "43,210");
        assert_eq!(group_thousands(1234567), "1,234,567");
    }

    #[test]
    fn price_simple() {
        let j = r#"{"bitcoin":{"usd":43210.5}}"#;
        assert_eq!(parse_price(j, "bitcoin", "usd"), Some(43210.5));
    }

    #[test]
    fn price_multi_and_missing() {
        let j = r#"{"bitcoin":{"usd":43210},"ethereum":{"usd":2310}}"#;
        assert_eq!(parse_price(j, "ethereum", "usd"), Some(2310.0));
        assert_eq!(parse_price(j, "dogecoin", "usd"), None);
        assert_eq!(parse_price(j, "bitcoin", "eur"), None);
    }

    #[test]
    fn widget_single() {
        let j = r#"{"bitcoin":{"usd":43210}}"#;
        assert_eq!(format_widget(j, &cfg(&["bitcoin"])), "$43,210");
    }

    #[test]
    fn widget_multi_labels_each() {
        let j = r#"{"bitcoin":{"usd":43210},"ethereum":{"usd":2310}}"#;
        assert_eq!(
            format_widget(j, &cfg(&["bitcoin", "ethereum"])),
            "bitcoin $43,210  ethereum $2,310"
        );
    }

    #[test]
    fn widget_empty_when_unconfigured() {
        assert_eq!(value(&cfg(&[])), "");
    }
}
