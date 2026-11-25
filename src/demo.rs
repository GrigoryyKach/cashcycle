//! `--demo`: plausible subscriptions and fixed exchange rates for screenshots. Nothing is read from or written to disk.

use crate::model::period::{Period, Unit};
use crate::model::rates::{self, Rates};
use crate::model::types::{new_id, Data, Pause, Subscription};
use chrono::{Days, NaiveDate};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::str::FromStr;

fn sub(name: &str, url: &str, price: &str, cur: &str, period: Period, due_in: u64, cat: &str, note: &str, today: NaiveDate) -> Subscription {
    Subscription {
        id: new_id(),
        name: name.into(),
        url: url.into(),
        price: Decimal::from_str(price).unwrap(),
        currency: cur.into(),
        period,
        start: today + Days::new(due_in),
        category: cat.into(),
        note: note.into(),
        paused: None,
    }
}

pub fn data(today: NaiveDate) -> Data {
    let mut d = Data::default();
    let c = |every, unit| Period::Custom { every, unit };
    d.subs = vec![
        sub("Netflix Premium", "https://netflix.com", "15.49", "USD", Period::Monthly, 3, "Video", "", today),
        sub("YouTube Premium", "https://youtube.com/premium", "159", "UAH", Period::Monthly, 6, "Video", "Family plan", today),
        sub("Spotify Duo", "https://spotify.com", "14.99", "EUR", Period::Monthly, 9, "Music", "", today),
        sub("iCloud+ 2 TB", "https://icloud.com", "9.99", "USD", Period::Monthly, 12, "Cloud", "", today),
        sub("Adobe Creative Cloud", "https://adobe.com", "60.49", "USD", Period::Monthly, 17, "Software", "", today),
        sub("JetBrains All Products", "https://jetbrains.com", "289", "EUR", Period::Yearly, 41, "Software", "Renews in January", today),
        sub("Xbox Game Pass", "https://xbox.com", "14.99", "USD", Period::Monthly, 21, "Gaming", "", today),
        sub("Gym", "", "900", "UAH", Period::Monthly, 1, "Other", "Paid at the desk", today),
        sub("Domain and hosting", "https://example.com", "36", "EUR", Period::Yearly, 74, "Cloud", "", today),
        sub("Coffee club", "", "12", "USD", c(2, Unit::Week), 5, "Other", "", today),
        sub("Disney+", "https://disneyplus.com", "7.99", "USD", Period::Monthly, 8, "Video", "", today),
        sub("Audible", "https://audible.com", "9.99", "EUR", Period::Monthly, 14, "Music", "", today),
    ];
    d.subs[10].paused = Some(Pause { since: today - Days::new(12), until: None });
    d.subs[11].paused = Some(Pause { since: today - Days::new(3), until: Some(today + Days::new(24)) });
    d.normalize();
    d
}

/// Fixed, believable rates (base USD), "updated" a few minutes ago.
pub fn rates() -> Rates {
    let mut m = BTreeMap::new();
    for (k, v) in [
        ("USD", 1.0), ("EUR", 0.92), ("UAH", 41.6), ("GBP", 0.79), ("PLN", 4.05), ("CZK", 23.2), ("CHF", 0.88), ("JPY", 151.0),
        ("CNY", 7.24), ("CAD", 1.37), ("AUD", 1.52), ("SEK", 10.6), ("NOK", 10.9), ("DKK", 6.87), ("TRY", 34.1), ("GEL", 2.7),
        ("KZT", 480.0), ("ILS", 3.7), ("INR", 83.5), ("BRL", 5.2),
    ] {
        m.insert(k.to_string(), v);
    }
    let now = rates::now_unix();
    Rates { base: "USD".into(), rates: m, as_of: now - 600, fetched: now - 600, source: "er-api".into() }
}
