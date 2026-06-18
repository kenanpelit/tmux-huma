//! Cryptocurrency prices via the CoinGecko public API.
//!
//! Fetched by the daemon but rate-limited by `@huma-kripto-ttl`: the value is
//! cached in `$XDG_RUNTIME_DIR/huma-kripto.cache` and only re-fetched once the
//! TTL elapses, so the daemon tick (seconds) never hammers the API. A failed
//! fetch falls back to the last good value instead of blanking the widget.
//!
//! Uses the `coins/markets` endpoint so each coin's ticker `symbol` comes back
//! with the price — rendered as a currency glyph (₿, Ξ, …) or the upper-case
//! ticker for coins without one.

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

/// Status-bar price: large coins as whole numbers with thousands separators,
/// mid-range with 2 decimals, sub-$1 coins with 4.
pub fn fmt_price(p: f64) -> String {
    if p >= 1000.0 {
        group_thousands(p.round() as u64)
    } else if p >= 1.0 {
        format!("{p:.2}")
    } else {
        format!("{p:.4}")
    }
}

/// A currency-style glyph for well-known tickers, else the upper-case ticker.
pub fn marker(symbol: &str) -> String {
    match symbol.to_lowercase().as_str() {
        "btc" => "₿",
        "eth" => "Ξ",
        "ltc" => "Ł",
        "doge" => "Ð",
        "xmr" => "ɱ",
        "dash" => "Đ",
        "usdt" | "usdc" => "₮",
        s => return s.to_uppercase(),
    }
    .to_string()
}

fn field_str(obj: &str, marker: &str) -> Option<String> {
    let after = &obj[obj.find(marker)? + marker.len()..];
    Some(after[..after.find('"')?].to_string())
}

fn field_num(obj: &str, marker: &str) -> Option<f64> {
    let after = &obj[obj.find(marker)? + marker.len()..];
    let num: String = after
        .trim_start_matches([':', ' '])
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num.parse().ok()
}

/// Pull a coin's `(symbol, price)` out of a CoinGecko `coins/markets` array. No
/// JSON crate: locate the coin's object by `id`, bound it to the next `id`, then
/// read its `symbol` + `current_price`.
pub fn parse_market(json: &str, coin: &str) -> Option<(String, f64)> {
    let key = format!("\"id\":\"{coin}\"");
    let start = json.find(&key)?;
    let rest = &json[start..];
    let end = rest[key.len()..]
        .find("\"id\":\"")
        .map(|i| i + key.len())
        .unwrap_or(rest.len());
    let obj = &rest[..end];
    let symbol = field_str(obj, "\"symbol\":\"")?;
    let price = field_num(obj, "\"current_price\":")?;
    Some((symbol, price))
}

/// Build the widget from a CoinGecko `coins/markets` response.
pub fn format_widget(json: &str, cfg: &Config) -> String {
    cfg.kripto_coins
        .iter()
        .filter_map(|coin| {
            parse_market(json, coin).map(|(sym, price)| {
                format!("{} {}{}", marker(&sym), cfg.kripto_symbol, fmt_price(price))
            })
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn fetch(cfg: &Config) -> Option<String> {
    let ids = cfg.kripto_coins.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/markets?vs_currency={}&ids={ids}",
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
        let mut c = Config::test();
        c.kripto_coins = coins.iter().map(|s| s.to_string()).collect();
        c
    }

    #[test]
    fn thousands() {
        assert_eq!(group_thousands(100), "100");
        assert_eq!(group_thousands(43210), "43,210");
        assert_eq!(group_thousands(1234567), "1,234,567");
    }

    #[test]
    fn price_formatting() {
        assert_eq!(fmt_price(62624.0), "62,624");
        assert_eq!(fmt_price(1681.66), "1,682");
        assert_eq!(fmt_price(12.5), "12.50");
        assert_eq!(fmt_price(0.1234), "0.1234");
    }

    #[test]
    fn markers() {
        assert_eq!(marker("btc"), "₿");
        assert_eq!(marker("ETH"), "Ξ");
        assert_eq!(marker("sol"), "SOL");
    }

    #[test]
    fn market_parse() {
        let j = r#"[{"id":"bitcoin","symbol":"btc","current_price":62624},{"id":"ethereum","symbol":"eth","current_price":1681.66}]"#;
        assert_eq!(parse_market(j, "bitcoin"), Some(("btc".into(), 62624.0)));
        assert_eq!(parse_market(j, "ethereum"), Some(("eth".into(), 1681.66)));
        assert_eq!(parse_market(j, "dogecoin"), None);
    }

    #[test]
    fn widget_single() {
        let j = r#"[{"id":"bitcoin","symbol":"btc","current_price":62624}]"#;
        assert_eq!(format_widget(j, &cfg(&["bitcoin"])), "₿ $62,624");
    }

    #[test]
    fn widget_multi() {
        let j = r#"[{"id":"bitcoin","symbol":"btc","current_price":62624},{"id":"ethereum","symbol":"eth","current_price":1681.66}]"#;
        assert_eq!(
            format_widget(j, &cfg(&["bitcoin", "ethereum"])),
            "₿ $62,624  Ξ $1,682"
        );
    }

    #[test]
    fn widget_empty_when_unconfigured() {
        assert_eq!(value(&cfg(&[])), "");
    }
}
