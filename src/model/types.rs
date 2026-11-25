//! The data model: subscriptions, pausing, categories and link validation.

use super::period::Period;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Version of `data.json`. 1 stores prices as decimal strings; version 0 (no field) stored plain numbers.
pub const SCHEMA_VERSION: u32 = 1;

/// Built-in categories. They are stored by these English keys and translated for display.
pub const BUILTIN_CATEGORIES: [&str; 6] = ["Video", "Music", "Software", "Cloud", "Gaming", "Other"];

/// Longest name, note and category the editor accepts.
pub const MAX_NAME: usize = 80;
pub const MAX_NOTE: usize = 500;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pause {
    pub since: NaiveDate,
    /// Resumes by itself on this day.
    #[serde(default)]
    pub until: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    /// Link to the service, empty or a normalised `https://` URL.
    #[serde(default)]
    pub url: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    pub currency: String,
    pub period: Period,
    /// The anchor: a known payment date (the next one, or any past one). Later payments are computed from it.
    pub start: NaiveDate,
    /// A built-in key, a user-defined name, or empty for none.
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub paused: Option<Pause>,
}

impl Subscription {
    pub fn is_paused(&self) -> bool {
        self.paused.is_some()
    }

    /// The next payment on or after `today`.
    pub fn next_payment(&self, today: NaiveDate) -> NaiveDate {
        self.period.next_on_or_after(self.start, today)
    }

    /// Price of one payment as a monthly cost, in the subscription's own currency.
    pub fn monthly(&self) -> Decimal {
        self.period.monthly(self.price)
    }

    pub fn yearly(&self) -> Decimal {
        self.period.yearly(self.price)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Data {
    pub version: u32,
    #[serde(default)]
    pub subs: Vec<Subscription>,
    /// User-defined categories (the built-in ones are not listed here).
    #[serde(default)]
    pub categories: Vec<String>,
}

impl Default for Data {
    fn default() -> Self {
        Data { version: SCHEMA_VERSION, subs: Vec::new(), categories: Vec::new() }
    }
}

impl Data {
    /// Cleans what a hand-edited or old file may contain.
    pub fn normalize(&mut self) {
        let mut seen = std::collections::HashSet::new();
        for s in &mut self.subs {
            s.name = clip(s.name.trim(), MAX_NAME);
            s.note = clip(s.note.trim(), MAX_NOTE);
            s.category = clip(s.category.trim(), MAX_NAME);
            s.currency = s.currency.trim().to_uppercase();
            s.period = s.period.sanitized();
            if s.price.is_sign_negative() {
                s.price = Decimal::ZERO;
            }
            if s.url.trim().is_empty() || normalize_url(&s.url).is_err() {
                s.url = String::new();
            }
            if s.id.is_empty() || !seen.insert(s.id.clone()) {
                s.id = new_id();
                seen.insert(s.id.clone());
            }
        }
        // Categories used by subscriptions but missing from the list are added, so they stay selectable.
        let used: Vec<String> = self.subs.iter().map(|s| s.category.clone()).collect();
        for c in used {
            self.add_category(&c);
        }
        self.categories.retain(|c| !c.trim().is_empty());
        self.categories.dedup();
        self.version = SCHEMA_VERSION;
    }

    /// Adds a user-defined category if it is new and not a built-in one. Returns its stored name.
    pub fn add_category(&mut self, name: &str) -> String {
        let name = clip(name.trim(), MAX_NAME);
        if name.is_empty() {
            return name;
        }
        if let Some(b) = BUILTIN_CATEGORIES.iter().find(|b| b.eq_ignore_ascii_case(&name)) {
            return b.to_string();
        }
        if let Some(existing) = self.categories.iter().find(|c| c.to_lowercase() == name.to_lowercase()) {
            return existing.clone();
        }
        self.categories.push(name.clone());
        name
    }

    /// Built-in categories first, then the user's own.
    pub fn all_categories(&self) -> Vec<String> {
        BUILTIN_CATEGORIES.iter().map(|s| s.to_string()).chain(self.categories.iter().cloned()).collect()
    }

    /// Stable colour slot for a category (its position in the list; none gets the neutral slot 255).
    pub fn category_slot(&self, name: &str) -> usize {
        if name.is_empty() {
            return usize::MAX;
        }
        self.all_categories().iter().position(|c| c == name).unwrap_or(usize::MAX)
    }

    pub fn sub(&self, id: &str) -> Option<&Subscription> {
        self.subs.iter().find(|s| s.id == id)
    }

    pub fn sub_mut(&mut self, id: &str) -> Option<&mut Subscription> {
        self.subs.iter_mut().find(|s| s.id == id)
    }

    pub fn active(&self) -> impl Iterator<Item = &Subscription> {
        self.subs.iter().filter(|s| !s.is_paused())
    }

    pub fn paused(&self) -> impl Iterator<Item = &Subscription> {
        self.subs.iter().filter(|s| s.is_paused())
    }

    /// Resumes every subscription whose "pause until" day has come. Returns how many woke up.
    /// The payment schedule stays anchored on the original date, so the next payment is simply the next one due.
    pub fn auto_resume(&mut self, today: NaiveDate) -> usize {
        let mut n = 0;
        for s in &mut self.subs {
            if s.paused.is_some_and(|p| p.until.is_some_and(|u| u <= today)) {
                s.paused = None;
                n += 1;
            }
        }
        n
    }

    /// Pauses a subscription (optionally until a day after today).
    pub fn pause(&mut self, id: &str, today: NaiveDate, until: Option<NaiveDate>) -> bool {
        let Some(s) = self.sub_mut(id) else { return false };
        let until = until.filter(|u| *u > today);
        s.paused = Some(Pause { since: today, until });
        true
    }

    /// Resumes a subscription. With `next_payment` the schedule is re-anchored on that day.
    pub fn resume(&mut self, id: &str, next_payment: Option<NaiveDate>) -> bool {
        let Some(s) = self.sub_mut(id) else { return false };
        s.paused = None;
        if let Some(d) = next_payment {
            s.start = d;
        }
        true
    }
}

fn clip(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A short unique id: time in microseconds plus a counter, in hex.
pub fn new_id() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_micros()).unwrap_or(0);
    format!("{:x}{:03x}", t, COUNTER.fetch_add(1, Ordering::Relaxed) % 4096)
}

/// Validates and normalises a link: `https://` is added when missing; only http(s) with a real host is accepted.
/// An empty input is fine and stays empty.
pub fn normalize_url(input: &str) -> Result<String, ()> {
    let s = input.trim();
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(());
    }
    let full = match s.split_once("://") {
        Some((scheme, _)) => {
            if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
                return Err(());
            }
            s.to_string()
        }
        None => format!("https://{s}"),
    };
    let rest = full.split_once("://").map(|x| x.1).unwrap_or("");
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.contains('@') {
        return Err(());
    }
    let host = match authority.rsplit_once(':') {
        Some((h, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => h,
        Some(_) => return Err(()),
        None => authority,
    };
    let labels: Vec<&str> = host.split('.').collect();
    let ok = labels.len() >= 2
        && labels.iter().all(|l| !l.is_empty() && !l.starts_with('-') && !l.ends_with('-') && l.chars().all(|c| c.is_alphanumeric() || c == '-'))
        && labels.last().is_some_and(|t| t.chars().count() >= 2);
    if !ok || full.len() > 300 {
        return Err(());
    }
    Ok(full)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::period::Unit;
    use std::str::FromStr;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    pub fn sample(name: &str, price: &str, cur: &str, period: Period) -> Subscription {
        Subscription {
            id: new_id(),
            name: name.into(),
            url: String::new(),
            price: Decimal::from_str(price).unwrap(),
            currency: cur.into(),
            period,
            start: d(2026, 11, 10),
            category: String::new(),
            note: String::new(),
            paused: None,
        }
    }

    #[test]
    fn url_validation() {
        assert_eq!(normalize_url(""), Ok(String::new()));
        assert_eq!(normalize_url("netflix.com").as_deref(), Ok("https://netflix.com"));
        assert_eq!(normalize_url("  www.spotify.com/account  ").as_deref(), Ok("https://www.spotify.com/account"));
        assert_eq!(normalize_url("http://example.org/a?b=1#c").as_deref(), Ok("http://example.org/a?b=1#c"));
        assert_eq!(normalize_url("https://example.com:8443/x").as_deref(), Ok("https://example.com:8443/x"));
        for bad in ["javascript:alert(1)", "ftp://example.com", "file:///c:/x", "no spaces.com", "localhost", "https://", "a@b.com", "https://user@example.com", "-x.com", "x..com", "https://x.c", "https://example.com:abc"] {
            assert!(normalize_url(bad).is_err(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn pause_and_resume_exclude_from_active() {
        let mut data = Data::default();
        let a = sample("A", "10", "USD", Period::Monthly);
        let b = sample("B", "5", "USD", Period::Monthly);
        let (ida, idb) = (a.id.clone(), b.id.clone());
        data.subs = vec![a, b];
        assert_eq!(data.active().count(), 2);
        assert!(data.pause(&ida, d(2026, 11, 25), None));
        assert_eq!(data.active().count(), 1);
        assert_eq!(data.paused().count(), 1);
        assert_eq!(data.active().next().unwrap().id, idb);
        assert!(data.resume(&ida, Some(d(2026, 12, 1))));
        assert_eq!(data.active().count(), 2);
        assert_eq!(data.sub(&ida).unwrap().start, d(2026, 12, 1));
        assert!(!data.pause("missing", d(2026, 11, 25), None));
    }

    #[test]
    fn pause_until_resumes_by_itself() {
        let mut data = Data::default();
        let a = sample("A", "10", "USD", Period::Monthly);
        let id = a.id.clone();
        data.subs = vec![a];
        let today = d(2026, 11, 25);
        data.pause(&id, today, Some(d(2026, 12, 5)));
        assert_eq!(data.auto_resume(d(2026, 12, 4)), 0);
        assert!(data.sub(&id).unwrap().is_paused());
        assert_eq!(data.auto_resume(d(2026, 12, 5)), 1);
        assert!(!data.sub(&id).unwrap().is_paused());
        // a past or same-day "until" is ignored: it would resume at once
        data.pause(&id, today, Some(today));
        assert_eq!(data.sub(&id).unwrap().paused.unwrap().until, None);
    }

    #[test]
    fn categories_are_unique_and_builtins_win() {
        let mut data = Data::default();
        assert_eq!(data.add_category("video"), "Video");
        assert_eq!(data.add_category("Books"), "Books");
        assert_eq!(data.add_category("books"), "Books");
        assert_eq!(data.add_category("  "), "");
        assert_eq!(data.categories, vec!["Books".to_string()]);
        assert_eq!(data.all_categories().len(), 7);
        assert_eq!(data.category_slot("Video"), 0);
        assert_eq!(data.category_slot("Books"), 6);
        assert_eq!(data.category_slot(""), usize::MAX);
    }

    #[test]
    fn normalize_cleans_hand_edited_data() {
        let mut data = Data::default();
        let mut a = sample("  Padded  ", "10", "usd", Period::Custom { every: 0, unit: Unit::Day });
        a.url = "javascript:x".into();
        a.category = "Mine".into();
        let mut b = sample("B", "-4", "UAH", Period::Monthly);
        b.id = a.id.clone();
        data.subs = vec![a, b];
        data.normalize();
        assert_eq!(data.subs[0].name, "Padded");
        assert_eq!(data.subs[0].currency, "USD");
        assert_eq!(data.subs[0].period, Period::Custom { every: 1, unit: Unit::Day });
        assert_eq!(data.subs[0].url, "");
        assert_ne!(data.subs[0].id, data.subs[1].id);
        assert!(data.subs[1].price.is_zero());
        assert!(data.categories.contains(&"Mine".to_string()));
    }

    #[test]
    fn ids_are_unique() {
        let ids: std::collections::HashSet<String> = (0..2000).map(|_| new_id()).collect();
        assert_eq!(ids.len(), 2000);
    }

    #[test]
    fn json_round_trip_keeps_exact_prices() {
        let mut data = Data::default();
        data.subs.push(sample("A", "9.99", "USD", Period::Yearly));
        let text = serde_json::to_string(&data).unwrap();
        assert!(text.contains("\"9.99\""));
        let back: Data = serde_json::from_str(&text).unwrap();
        assert_eq!(back, data);
    }
}
