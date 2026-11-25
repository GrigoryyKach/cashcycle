//! Exchange rates: one base currency, cross-rates, an on-disk cache, and two free sources without an API key.
//! See `docs/RATES.md` for the terms of each source.

use super::money;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

pub const CACHE_FILE: &str = "rates.json";
/// Rates older than this are refreshed on start.
pub const MAX_AGE_SECS: i64 = 6 * 3600;

pub const ER_API_URL: &str = "https://open.er-api.com/v6/latest/USD";
pub const NBU_URL: &str = "https://bank.gov.ua/NBUStatService/v1/statdirectory/exchange?json";
#[allow(dead_code)]
/// The attribution text the free ExchangeRate-API plan asks for.
pub const ATTRIBUTION: &str = "Rates by Exchange Rate API (exchangerate-api.com)";
pub const ATTRIBUTION_URL: &str = "https://www.exchangerate-api.com";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Rates {
    /// The base currency: `rates[base] == 1`.
    pub base: String,
    /// Units of each currency per one unit of `base`.
    pub rates: BTreeMap<String, f64>,
    /// When the provider says the figures are from (unix seconds).
    pub as_of: i64,
    /// When this app downloaded them (unix seconds).
    pub fetched: i64,
    /// `er-api` or `nbu`.
    pub source: String,
}

impl Rates {
    pub fn rate(&self, code: &str) -> Option<f64> {
        self.rates.get(&code.to_uppercase()).copied().filter(|r| r.is_finite() && *r > 0.0)
    }

    /// Units of `to` per one unit of `from`, through the base currency.
    pub fn cross(&self, from: &str, to: &str) -> Option<f64> {
        if from.eq_ignore_ascii_case(to) {
            return Some(1.0);
        }
        Some(self.rate(to)? / self.rate(from)?)
    }

    /// Converts an amount. `None` when either currency has no rate.
    pub fn convert(&self, amount: Decimal, from: &str, to: &str) -> Option<Decimal> {
        if from.eq_ignore_ascii_case(to) {
            return Some(amount);
        }
        money::from_f64(money::to_f64(amount) * self.cross(from, to)?)
    }

    pub fn age_secs(&self, now: i64) -> i64 {
        (now - self.fetched).max(0)
    }

    pub fn is_stale(&self, now: i64) -> bool {
        self.age_secs(now) > MAX_AGE_SECS
    }
}

/// Converts with optional rates: the same currency never needs any.
pub fn convert(rates: Option<&Rates>, amount: Decimal, from: &str, to: &str) -> Option<Decimal> {
    if from.eq_ignore_ascii_case(to) {
        return Some(amount);
    }
    rates?.convert(amount, from, to)
}

/// Drops currencies the app never converts (see `currency::EXCLUDED`).
fn strip_excluded(rates: &mut BTreeMap<String, f64>) {
    for code in super::currency::EXCLUDED {
        rates.remove(code);
    }
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

// ───────────────────────── parsing ─────────────────────────

/// `open.er-api.com/v6/latest/USD`.
pub fn parse_er_api(text: &str, now: i64) -> Option<Rates> {
    let v: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    if v.get("result")?.as_str()? != "success" {
        return None;
    }
    let base = v.get("base_code")?.as_str()?.to_uppercase();
    let mut rates = BTreeMap::new();
    for (k, val) in v.get("rates")?.as_object()? {
        if let Some(r) = val.as_f64().filter(|r| r.is_finite() && *r > 0.0) {
            rates.insert(k.to_uppercase(), r);
        }
    }
    strip_excluded(&mut rates);
    if rates.get(&base).copied() != Some(1.0) || rates.len() < 20 {
        return None;
    }
    let as_of = v.get("time_last_update_unix").and_then(|t| t.as_i64()).filter(|t| *t > 0).unwrap_or(now);
    Some(Rates { base, rates, as_of, fetched: now, source: "er-api".into() })
}

/// The National Bank of Ukraine: a list of `{cc, rate}` with the official hryvnia value of one unit.
/// The base becomes UAH.
pub fn parse_nbu(text: &str, now: i64) -> Option<Rates> {
    let v: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    let mut rates = BTreeMap::new();
    rates.insert("UAH".to_string(), 1.0);
    let mut as_of = now;
    for item in v.as_array()? {
        let cc = item.get("cc").and_then(|c| c.as_str()).unwrap_or("").to_uppercase();
        let uah = item.get("rate").and_then(|r| r.as_f64()).unwrap_or(0.0);
        if cc.len() == 3 && cc.chars().all(|c| c.is_ascii_uppercase()) && uah.is_finite() && uah > 0.0 && cc != "UAH" {
            rates.insert(cc, 1.0 / uah);
        }
        if let Some(d) = item.get("exchangedate").and_then(|d| d.as_str()).and_then(|d| chrono::NaiveDate::parse_from_str(d, "%d.%m.%Y").ok()) {
            as_of = d.and_hms_opt(0, 0, 0).map(|t| t.and_utc().timestamp()).unwrap_or(now);
        }
    }
    strip_excluded(&mut rates);
    if rates.len() < 10 || !rates.contains_key("USD") || !rates.contains_key("EUR") {
        return None;
    }
    Some(Rates { base: "UAH".into(), rates, as_of, fetched: now, source: "nbu".into() })
}

// ───────────────────────── network ─────────────────────────

fn http_get(url: &str) -> Result<String, String> {
    if std::env::var_os("CASHCYCLE_OFFLINE").is_some_and(|v| !v.is_empty()) {
        return Err("offline (CASHCYCLE_OFFLINE)".into());
    }
    let mut cmd = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    cmd.args(["-fsSL", "--max-time", "15", "--proto", "=https", "--proto-redir", "=https", "-A", "Cashcycle"]).arg(url);
    cmd.stdin(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let out = cmd.output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("HTTP request failed ({})", out.status));
    }
    String::from_utf8(out.stdout).map_err(|e| e.to_string())
}

/// Downloads fresh rates: ExchangeRate-API first, the National Bank of Ukraine as the fallback.
pub fn fetch() -> Result<Rates, String> {
    let now = now_unix();
    let first = http_get(ER_API_URL).and_then(|t| parse_er_api(&t, now).ok_or_else(|| "unexpected answer from open.er-api.com".to_string()));
    match first {
        Ok(r) => Ok(r),
        Err(e1) => http_get(NBU_URL)
            .and_then(|t| parse_nbu(&t, now).ok_or_else(|| "unexpected answer from bank.gov.ua".to_string()))
            .map_err(|e2| format!("{e1}; {e2}")),
    }
}

// ───────────────────────── cache ─────────────────────────

pub fn load_cache(dir: &Path) -> Option<Rates> {
    let text = std::fs::read_to_string(dir.join(CACHE_FILE)).ok()?;
    let mut r: Rates = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    strip_excluded(&mut r.rates);
    (r.rates.len() >= 2 && r.rate(&r.base) == Some(1.0)).then_some(r)
}

pub fn save_cache(dir: &Path, r: &Rates) -> bool {
    serde_json::to_vec(r).ok().is_some_and(|b| super::storage::write_atomic(&dir.join(CACHE_FILE), &b).is_ok())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    pub fn sample() -> Rates {
        let mut rates = BTreeMap::new();
        for (k, v) in [("USD", 1.0), ("EUR", 0.8), ("UAH", 40.0), ("JPY", 150.0), ("GBP", 0.5)] {
            rates.insert(k.to_string(), v);
        }
        Rates { base: "USD".into(), rates, as_of: 1_000, fetched: 2_000, source: "er-api".into() }
    }

    #[test]
    fn cross_rates_go_through_the_base() {
        let r = sample();
        assert_eq!(r.cross("USD", "UAH"), Some(40.0));
        assert_eq!(r.cross("UAH", "USD"), Some(1.0 / 40.0));
        assert!((r.cross("EUR", "UAH").unwrap() - 50.0).abs() < 1e-9);
        assert!((r.cross("UAH", "EUR").unwrap() - 0.02).abs() < 1e-9);
        assert_eq!(r.cross("EUR", "eur"), Some(1.0));
        assert_eq!(r.cross("EUR", "XYZ"), None);
        assert_eq!(r.cross("XYZ", "EUR"), None);
    }

    #[test]
    fn conversion_and_rounding() {
        let r = sample();
        let eur = r.convert(dec("10"), "USD", "EUR").unwrap();
        assert_eq!(money::round(eur, 2), dec("8.00"));
        let uah = r.convert(dec("9.99"), "USD", "UAH").unwrap();
        assert_eq!(money::round(uah, 2), dec("399.60"));
        let jpy = r.convert(dec("9.99"), "USD", "JPY").unwrap();
        assert_eq!(money::round(jpy, 0), dec("1498"));
        // a figure that sits exactly halfway rounds to even
        let half = r.convert(dec("2.5"), "USD", "USD").unwrap();
        assert_eq!(money::round(half, 0), dec("2"));
    }

    #[test]
    fn missing_rate_gives_none_never_a_wrong_number() {
        let r = sample();
        assert_eq!(r.convert(dec("5"), "USD", "XYZ"), None);
        assert_eq!(r.convert(dec("5"), "XYZ", "USD"), None);
        // without any rates only the same currency converts
        assert_eq!(convert(None, dec("5"), "USD", "USD"), Some(dec("5")));
        assert_eq!(convert(None, dec("5"), "USD", "EUR"), None);
        assert_eq!(convert(Some(&r), dec("5"), "USD", "EUR").is_some(), true);
    }

    #[test]
    fn staleness() {
        let r = sample();
        assert!(!r.is_stale(2_000 + MAX_AGE_SECS));
        assert!(r.is_stale(2_000 + MAX_AGE_SECS + 1));
        assert_eq!(r.age_secs(100), 0);
    }

    fn er_api_text() -> String {
        let mut rates = String::new();
        for i in 0..30 {
            rates.push_str(&format!("\"C{}{}\":{}.5,", (b'A' + (i / 26) as u8) as char, (b'A' + (i % 26) as u8) as char, i + 1));
        }
        format!(
            "{{\"result\":\"success\",\"base_code\":\"USD\",\"time_last_update_unix\":1795564951,\"rates\":{{{rates}\"USD\":1,\"UAH\":45.326038,\"EUR\":0.91245}}}}"
        )
    }

    #[test]
    fn parses_the_er_api_answer() {
        let r = parse_er_api(&er_api_text(), 5_000).unwrap();
        assert_eq!(r.base, "USD");
        assert_eq!(r.rate("uah"), Some(45.326038));
        assert_eq!(r.as_of, 1795564951);
        assert_eq!(r.fetched, 5_000);
        assert_eq!(r.source, "er-api");
        // failures and nonsense are refused
        let with_rub = er_api_text().replace("\"USD\":1,", "\"USD\":1,\"RUB\":90.5,\"BYN\":3.2,");
        let r2 = parse_er_api(&with_rub, 1).unwrap();
        assert!(r2.rate("RUB").is_none() && r2.rate("BYN").is_none());
        assert!(r2.convert(dec("5"), "RUB", "USD").is_none());
        assert!(parse_er_api("{\"result\":\"error\",\"error-type\":\"unsupported-code\"}", 1).is_none());
        assert!(parse_er_api("not json", 1).is_none());
        assert!(parse_er_api("{\"result\":\"success\",\"base_code\":\"USD\",\"rates\":{\"USD\":1}}", 1).is_none());
        assert!(parse_er_api(&er_api_text().replace("\"USD\":1,", "\"USD\":2,"), 1).is_none());
    }

    #[test]
    fn parses_the_nbu_answer_with_uah_as_base() {
        let mut items = String::new();
        for (i, cc) in ["AUD", "CAD", "CHF", "CZK", "GBP", "JPY", "PLN", "SEK", "NOK", "TRY"].iter().enumerate() {
            items.push_str(&format!("{{\"r030\":{i},\"txt\":\"x\",\"rate\":{}.5,\"cc\":\"{cc}\",\"exchangedate\":\"25.11.2026\"}},", i + 1));
        }
        let text = format!(
            "[{items}{{\"r030\":840,\"txt\":\"Долар\",\"rate\":41.5,\"cc\":\"USD\",\"exchangedate\":\"25.11.2026\"}},{{\"r030\":978,\"txt\":\"Євро\",\"rate\":48.25,\"cc\":\"EUR\",\"exchangedate\":\"25.11.2026\"}},{{\"r030\":959,\"txt\":\"Золото\",\"rate\":1.0,\"cc\":\"XAU\",\"exchangedate\":\"25.11.2026\"}}]"
        );
        let r = parse_nbu(&text, 7).unwrap();
        assert_eq!(r.base, "UAH");
        assert_eq!(r.rate("UAH"), Some(1.0));
        let usd_in_uah = r.convert(dec("1"), "USD", "UAH").unwrap();
        assert!((money::to_f64(usd_in_uah) - 41.5).abs() < 1e-9);
        let eur_in_usd = r.cross("EUR", "USD").unwrap();
        assert!((eur_in_usd - 48.25 / 41.5).abs() < 1e-9);
        assert_eq!(r.source, "nbu");
        assert!(r.as_of > 1_700_000_000);
        assert!(parse_nbu("[]", 1).is_none());
        assert!(parse_nbu("{}", 1).is_none());
    }

    #[test]
    fn cache_round_trip_and_bad_cache() {
        let dir = std::env::temp_dir().join(format!("cashcycle-rates-{}", crate::model::types::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load_cache(&dir).is_none());
        let r = sample();
        assert!(save_cache(&dir, &r));
        assert_eq!(load_cache(&dir), Some(r));
        std::fs::write(dir.join(CACHE_FILE), "{ broken").unwrap();
        assert!(load_cache(&dir).is_none());
        std::fs::write(dir.join(CACHE_FILE), "{\"base\":\"USD\",\"rates\":{\"USD\":2.0,\"EUR\":1.0},\"as_of\":1,\"fetched\":1,\"source\":\"x\"}").unwrap();
        assert!(load_cache(&dir).is_none(), "base must be 1");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn offline_flag_makes_the_fetch_fail_without_network() {
        // SAFETY: tests in this module that touch the variable are this one only.
        unsafe { std::env::set_var("CASHCYCLE_OFFLINE", "1") };
        let e = fetch().unwrap_err();
        unsafe { std::env::remove_var("CASHCYCLE_OFFLINE") };
        assert!(e.contains("offline"));
    }
}
