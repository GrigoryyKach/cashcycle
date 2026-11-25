//! Totals, upcoming payments and the category breakdown. Everything here is pure, so it is tested without a window.

use super::rates::{self, Rates};
use super::types::{Data, Subscription};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// How far ahead the "upcoming" list looks.
pub const UPCOMING_DAYS: i64 = 30;

/// Money of active subscriptions, in the chosen currency when the rates allow it.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Totals {
    /// Unrounded monthly total in the target currency (0 when `converted` is false).
    pub monthly: Decimal,
    pub yearly: Decimal,
    /// False when at least one currency differs from the target and there are no rates at all.
    pub converted: bool,
    /// Currencies that have no rate (their subscriptions are left out of `monthly` and `yearly`).
    pub missing: Vec<String>,
    /// Monthly and yearly totals per currency, always available.
    pub per_currency: Vec<(String, Decimal, Decimal)>,
    pub active: usize,
    pub paused: usize,
}

pub fn totals(data: &Data, target: &str, rates: Option<&Rates>) -> Totals {
    let mut t = Totals { converted: true, active: data.active().count(), paused: data.paused().count(), ..Default::default() };
    let mut per: BTreeMap<String, (Decimal, Decimal)> = BTreeMap::new();
    for s in data.active() {
        let e = per.entry(s.currency.clone()).or_default();
        e.0 += s.monthly();
        e.1 += s.yearly();
    }
    let needs_rates = per.keys().any(|c| !c.eq_ignore_ascii_case(target));
    if needs_rates && rates.is_none() {
        t.converted = false;
    } else {
        for (code, (m, y)) in &per {
            match (rates::convert(rates, *m, code, target), rates::convert(rates, *y, code, target)) {
                (Some(m), Some(y)) => {
                    t.monthly += m;
                    t.yearly += y;
                }
                _ => t.missing.push(code.clone()),
            }
        }
    }
    t.per_currency = per.into_iter().map(|(c, (m, y))| (c, m, y)).collect();
    t.per_currency.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    t
}

/// One payment coming up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upcoming {
    pub date: NaiveDate,
    pub id: String,
}

/// Payments of active subscriptions from `today` through the next 30 days, soonest first.
pub fn upcoming(data: &Data, today: NaiveDate) -> Vec<Upcoming> {
    let end = today + chrono::Days::new(UPCOMING_DAYS as u64);
    let mut out: Vec<Upcoming> = data
        .active()
        .flat_map(|s| s.period.between(s.start, today, end).into_iter().map(move |date| Upcoming { date, id: s.id.clone() }))
        .collect();
    out.sort_by(|a, b| {
        let na = data.sub(&a.id).map(|s| s.name.to_lowercase()).unwrap_or_default();
        let nb = data.sub(&b.id).map(|s| s.name.to_lowercase()).unwrap_or_default();
        a.date.cmp(&b.date).then(na.cmp(&nb))
    });
    out
}

/// Monthly cost per category in the target currency, biggest first. Subscriptions without a usable rate are skipped.
/// The empty name is "no category".
pub fn by_category(data: &Data, target: &str, rates: Option<&Rates>) -> Vec<(String, Decimal)> {
    let mut map: BTreeMap<String, Decimal> = BTreeMap::new();
    for s in data.active() {
        if let Some(v) = rates::convert(rates, s.monthly(), &s.currency, target) {
            *map.entry(s.category.clone()).or_default() += v;
        }
    }
    let mut v: Vec<(String, Decimal)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

/// A subscription's monthly cost in the target currency (for sorting and display).
pub fn monthly_in(s: &Subscription, target: &str, rates: Option<&Rates>) -> Option<Decimal> {
    rates::convert(rates, s.monthly(), &s.currency, target)
}

/// One payment's price in the target currency.
pub fn price_in(s: &Subscription, target: &str, rates: Option<&Rates>) -> Option<Decimal> {
    rates::convert(rates, s.price, &s.currency, target)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    Next,
    Price,
    Name,
}

/// Sorted indexes of the subscriptions (active or paused) that match the search text.
pub fn list(data: &Data, paused: bool, query: &str, sort: Sort, today: NaiveDate, target: &str, rates: Option<&Rates>) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    let mut idx: Vec<usize> = data
        .subs
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_paused() == paused)
        .filter(|(_, s)| q.is_empty() || s.name.to_lowercase().contains(&q) || s.category.to_lowercase().contains(&q) || s.note.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect();
    let name = |i: usize| data.subs[i].name.to_lowercase();
    match sort {
        Sort::Next => idx.sort_by(|a, b| data.subs[*a].next_payment(today).cmp(&data.subs[*b].next_payment(today)).then(name(*a).cmp(&name(*b)))),
        // Most expensive first; subscriptions without a rate go last.
        Sort::Price => idx.sort_by(|a, b| {
            let (pa, pb) = (monthly_in(&data.subs[*a], target, rates), monthly_in(&data.subs[*b], target, rates));
            match (pa, pb) {
                (Some(x), Some(y)) => y.cmp(&x).then(name(*a).cmp(&name(*b))),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => name(*a).cmp(&name(*b)),
            }
        }),
        Sort::Name => idx.sort_by_key(|i| name(*i)),
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::period::{Period, Unit};
    use crate::model::rates::tests::sample as sample_rates;
    use crate::model::types::tests::sample;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn data3() -> Data {
        let mut data = Data::default();
        data.subs.push(sample("Netflix", "10", "USD", Period::Monthly));
        data.subs.push(sample("Domain", "120", "EUR", Period::Yearly));
        data.subs.push(sample("Cloud", "400", "UAH", Period::Monthly));
        data
    }

    #[test]
    fn totals_in_one_currency_need_no_rates() {
        let mut data = Data::default();
        data.subs.push(sample("A", "10", "USD", Period::Monthly));
        data.subs.push(sample("B", "120", "USD", Period::Yearly));
        data.subs.push(sample("C", "12", "USD", Period::Weekly));
        let t = totals(&data, "USD", None);
        assert!(t.converted && t.missing.is_empty());
        assert_eq!(t.monthly, dec("72")); // 10 + 10 + 52
        assert_eq!(t.yearly, dec("864"));
        assert_eq!((t.active, t.paused), (3, 0));
    }

    #[test]
    fn totals_convert_through_the_rates() {
        let r = sample_rates(); // USD 1, EUR 0.8, UAH 40
        let t = totals(&data3(), "USD", Some(&r));
        // 10 USD + 10 EUR (=12.5 USD) + 400 UAH (=10 USD)
        assert!(t.converted);
        assert_eq!(crate::model::money::round(t.monthly, 2), dec("32.50"));
        assert_eq!(crate::model::money::round(t.yearly, 2), dec("390.00"));
        let u = totals(&data3(), "UAH", Some(&r));
        assert_eq!(crate::model::money::round(u.monthly, 2), dec("1300.00"));
        assert_eq!(t.per_currency.len(), 3);
    }

    #[test]
    fn offline_without_a_cache_shows_per_currency_not_a_wrong_total() {
        let t = totals(&data3(), "USD", None);
        assert!(!t.converted);
        assert!(t.monthly.is_zero() && t.yearly.is_zero());
        assert_eq!(t.per_currency.len(), 3);
        let eur = t.per_currency.iter().find(|c| c.0 == "EUR").unwrap();
        assert_eq!(eur.1, dec("10"));
        assert_eq!(eur.2, dec("120"));
    }

    #[test]
    fn a_currency_without_a_rate_is_reported_not_guessed() {
        let mut data = data3();
        data.subs.push(sample("Odd", "5", "XYZ", Period::Monthly));
        let t = totals(&data, "USD", Some(&sample_rates()));
        assert!(t.converted);
        assert_eq!(t.missing, vec!["XYZ".to_string()]);
        assert_eq!(crate::model::money::round(t.monthly, 2), dec("32.50"));
    }

    #[test]
    fn paused_subscriptions_are_excluded() {
        let mut data = data3();
        let id = data.subs[0].id.clone();
        data.pause(&id, d(2026, 11, 25), None);
        let t = totals(&data, "USD", Some(&sample_rates()));
        assert_eq!((t.active, t.paused), (2, 1));
        assert_eq!(crate::model::money::round(t.monthly, 2), dec("22.50"));
        assert!(by_category(&data, "USD", Some(&sample_rates())).iter().map(|c| c.1).sum::<Decimal>() < dec("22.51"));
        assert!(upcoming(&data, d(2026, 11, 25)).iter().all(|u| u.id != id));
        assert!(list(&data, false, "", Sort::Name, d(2026, 11, 25), "USD", None).iter().all(|i| data.subs[*i].id != id));
        assert_eq!(list(&data, true, "", Sort::Name, d(2026, 11, 25), "USD", None).len(), 1);
    }

    #[test]
    fn upcoming_covers_30_days_and_repeats_weekly_payments() {
        let mut data = Data::default();
        let mut a = sample("Weekly", "3", "USD", Period::Weekly);
        a.start = d(2026, 11, 26);
        let mut b = sample("Monthly", "10", "USD", Period::Monthly);
        b.start = d(2026, 12, 25);
        let mut c = sample("Yearly", "100", "USD", Period::Yearly);
        c.start = d(2027, 3, 1);
        let mut e = sample("Today", "1", "USD", Period::Custom { every: 2, unit: Unit::Month });
        e.start = d(2026, 11, 25);
        data.subs = vec![a, b, c, e];
        let up = upcoming(&data, d(2026, 11, 25));
        let names: Vec<(String, NaiveDate)> = up.iter().map(|u| (data.sub(&u.id).unwrap().name.clone(), u.date)).collect();
        assert_eq!(names[0], ("Today".to_string(), d(2026, 11, 25)));
        assert_eq!(names[1], ("Weekly".to_string(), d(2026, 11, 26)));
        assert_eq!(names.iter().filter(|n| n.0 == "Weekly").count(), 5); // 26 Nov, 3, 10, 17, 24 Dec
        assert_eq!(names.last().unwrap(), &("Monthly".to_string(), d(2026, 12, 25)));
        assert!(names.iter().all(|n| n.0 != "Yearly"));
        assert!(names.windows(2).all(|w| w[0].1 <= w[1].1));
    }

    #[test]
    fn category_breakdown_is_sorted_and_converted() {
        let mut data = data3();
        data.subs[0].category = "Video".into();
        data.subs[1].category = "Software".into();
        data.subs[2].category = "Video".into();
        let cats = by_category(&data, "USD", Some(&sample_rates()));
        assert_eq!(cats[0].0, "Video");
        assert_eq!(crate::model::money::round(cats[0].1, 2), dec("20.00"));
        assert_eq!(cats[1].0, "Software");
        // no rates: only same-currency entries can be counted
        let none = by_category(&data, "USD", None);
        assert_eq!(none.len(), 1);
    }

    #[test]
    fn sorting_search_and_filtering() {
        let mut data = data3();
        data.subs[0].start = d(2026, 12, 20);
        data.subs[1].start = d(2026, 12, 1);
        data.subs[2].start = d(2026, 11, 28);
        data.subs[2].note = "backup space".into();
        data.subs[0].price = dec("9");
        let today = d(2026, 11, 25);
        let r = sample_rates();
        let name = |idx: Vec<usize>| idx.iter().map(|i| data.subs[*i].name.clone()).collect::<Vec<_>>();
        assert_eq!(name(list(&data, false, "", Sort::Next, today, "USD", Some(&r))), ["Cloud", "Domain", "Netflix"]);
        assert_eq!(name(list(&data, false, "", Sort::Name, today, "USD", Some(&r))), ["Cloud", "Domain", "Netflix"]);
        // converted monthly: Netflix 10, Domain 12.5, Cloud 10 -> Domain first, then Cloud/Netflix by name
        assert_eq!(name(list(&data, false, "", Sort::Price, today, "USD", Some(&r))), ["Domain", "Cloud", "Netflix"]);
        assert_eq!(name(list(&data, false, "FLIX", Sort::Name, today, "USD", Some(&r))), ["Netflix"]);
        assert_eq!(name(list(&data, false, "space", Sort::Name, today, "USD", Some(&r))), ["Cloud"]);
        assert!(list(&data, false, "zzz", Sort::Name, today, "USD", Some(&r)).is_empty());
    }
}
