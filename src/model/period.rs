//! Billing periods: how often a subscription charges, how that normalises to a month, and the payment dates.
//!
//! Payment dates are always computed from the anchor (the date the person entered), never from the previous
//! payment: Jan 31 monthly gives Feb 28, Mar 31, Apr 30 and not Feb 28, Mar 28, Apr 28.

use chrono::{Datelike, Months, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Largest "every N" a custom period accepts.
pub const MAX_EVERY: u32 = 999;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Day,
    Week,
    Month,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom { every: u32, unit: Unit },
}

/// The kinds offered in the period dropdown (custom carries its own numbers).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom,
}

impl Kind {
    pub const ALL: [Kind; 5] = [Kind::Weekly, Kind::Monthly, Kind::Quarterly, Kind::Yearly, Kind::Custom];
}

/// A step between two payments.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Step {
    Days(u32),
    Months(u32),
}

impl Period {
    pub fn kind(self) -> Kind {
        match self {
            Period::Weekly => Kind::Weekly,
            Period::Monthly => Kind::Monthly,
            Period::Quarterly => Kind::Quarterly,
            Period::Yearly => Kind::Yearly,
            Period::Custom { .. } => Kind::Custom,
        }
    }

    pub fn of_kind(kind: Kind, every: u32, unit: Unit) -> Period {
        match kind {
            Kind::Weekly => Period::Weekly,
            Kind::Monthly => Period::Monthly,
            Kind::Quarterly => Period::Quarterly,
            Kind::Yearly => Period::Yearly,
            Kind::Custom => Period::Custom { every: every.clamp(1, MAX_EVERY), unit },
        }
    }

    /// Clamps a custom "every N" into 1..=999 (data files can be edited by hand).
    pub fn sanitized(self) -> Period {
        match self {
            Period::Custom { every, unit } => Period::Custom { every: every.clamp(1, MAX_EVERY), unit },
            p => p,
        }
    }

    fn step(self) -> Step {
        match self.sanitized() {
            Period::Weekly => Step::Days(7),
            Period::Monthly => Step::Months(1),
            Period::Quarterly => Step::Months(3),
            Period::Yearly => Step::Months(12),
            Period::Custom { every, unit } => match unit {
                Unit::Day => Step::Days(every),
                Unit::Week => Step::Days(7 * every),
                Unit::Month => Step::Months(every),
            },
        }
    }

    /// How many payments fall into a year. A year is 12 months or 52 weeks (364 days), so
    /// "every 7 days" and "weekly" normalise to exactly the same monthly figure (x 52 / 12).
    pub fn per_year(self) -> Decimal {
        match self.step() {
            Step::Days(n) => Decimal::from(364) / Decimal::from(n),
            Step::Months(n) => Decimal::from(12) / Decimal::from(n),
        }
    }

    /// The price of one payment as a monthly cost.
    pub fn monthly(self, price: Decimal) -> Decimal {
        price * self.per_year() / Decimal::from(12)
    }

    /// The price of one payment as a yearly cost.
    pub fn yearly(self, price: Decimal) -> Decimal {
        price * self.per_year()
    }

    /// The `k`-th payment counted from the anchor (`k = 0` is the anchor itself).
    pub fn occurrence(self, anchor: NaiveDate, k: u32) -> Option<NaiveDate> {
        match self.step() {
            Step::Days(n) => anchor.checked_add_days(chrono::Days::new(u64::from(n) * u64::from(k))),
            Step::Months(n) => anchor.checked_add_months(Months::new(n.checked_mul(k)?)),
        }
    }

    /// The first payment on or after `from`. An anchor in the future is itself the next payment.
    pub fn next_on_or_after(self, anchor: NaiveDate, from: NaiveDate) -> NaiveDate {
        if anchor >= from {
            return anchor;
        }
        let gap_days = (from - anchor).num_days().max(0) as u64;
        // Start a little before the estimate so the loop only walks forward.
        let mut k = match self.step() {
            Step::Days(n) => gap_days / u64::from(n),
            Step::Months(n) => (gap_days / 31) / u64::from(n),
        } as u32;
        loop {
            match self.occurrence(anchor, k) {
                Some(d) if d >= from => return d,
                Some(_) => k += 1,
                None => return from, // beyond the calendar: cannot happen for sane input
            }
        }
    }

    /// Every payment from `from` to `to` inclusive.
    pub fn between(self, anchor: NaiveDate, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        let mut out = Vec::new();
        let mut d = self.next_on_or_after(anchor, from);
        let mut guard = 0;
        while d <= to && guard < 400 {
            out.push(d);
            // Walk by index so month ends do not drift.
            let k = self.index_of(anchor, d);
            match self.occurrence(anchor, k + 1) {
                Some(n) if n > d => d = n,
                _ => break,
            }
            guard += 1;
        }
        out
    }

    /// Index of an occurrence date (`d` must be an occurrence at or after the anchor).
    fn index_of(self, anchor: NaiveDate, d: NaiveDate) -> u32 {
        if d <= anchor {
            return 0;
        }
        let gap = (d - anchor).num_days() as u64;
        let mut k = match self.step() {
            Step::Days(n) => gap / u64::from(n),
            Step::Months(n) => ((d.year() - anchor.year()) as i64 * 12 + d.month() as i64 - anchor.month() as i64).max(0) as u64 / u64::from(n),
        } as u32;
        // The estimate can be one off around month ends.
        while k > 0 && self.occurrence(anchor, k).is_some_and(|x| x > d) {
            k -= 1;
        }
        while self.occurrence(anchor, k).is_some_and(|x| x < d) {
            k += 1;
        }
        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }
    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn monthly_normalisation_of_every_period() {
        let ten = dec("10");
        assert_eq!(Period::Monthly.monthly(ten), dec("10"));
        assert_eq!(Period::Quarterly.monthly(dec("30")), dec("10"));
        assert_eq!(Period::Yearly.monthly(dec("120")), dec("10"));
        // weekly x 52 / 12
        let w = Period::Weekly.monthly(dec("12"));
        assert_eq!(w, dec("52"));
        assert_eq!(Period::Weekly.yearly(dec("12")), dec("624"));
        // custom periods follow the same rule
        assert_eq!(Period::Custom { every: 2, unit: Unit::Month }.monthly(dec("20")), dec("10"));
        assert_eq!(Period::Custom { every: 2, unit: Unit::Week }.monthly(dec("12")), dec("26"));
        assert_eq!(Period::Custom { every: 7, unit: Unit::Day }.monthly(dec("12")), Period::Weekly.monthly(dec("12")));
        assert_eq!(Period::Custom { every: 14, unit: Unit::Day }.monthly(dec("12")), dec("26"));
        assert_eq!(Period::Custom { every: 6, unit: Unit::Month }.yearly(dec("50")), dec("100"));
        assert_eq!(Period::Custom { every: 1, unit: Unit::Month }.yearly(dec("5")), dec("60"));
        // a full year of a monthly price equals the yearly price
        assert_eq!(Period::Yearly.yearly(dec("99.90")), dec("99.90"));
    }

    #[test]
    fn month_end_rolling_does_not_drift() {
        let a = d(2026, 1, 31);
        let p = Period::Monthly;
        assert_eq!(p.occurrence(a, 1), Some(d(2026, 2, 28)));
        assert_eq!(p.occurrence(a, 2), Some(d(2026, 3, 31)));
        assert_eq!(p.occurrence(a, 3), Some(d(2026, 4, 30)));
        assert_eq!(p.next_on_or_after(a, d(2026, 3, 1)), d(2026, 3, 31));
        assert_eq!(p.next_on_or_after(a, d(2026, 2, 1)), d(2026, 2, 28));
    }

    #[test]
    fn leap_years() {
        assert_eq!(Period::Monthly.occurrence(d(2028, 1, 31), 1), Some(d(2028, 2, 29)));
        assert_eq!(Period::Yearly.occurrence(d(2028, 2, 29), 1), Some(d(2029, 2, 28)));
        assert_eq!(Period::Yearly.occurrence(d(2028, 2, 29), 4), Some(d(2032, 2, 29)));
        assert_eq!(Period::Quarterly.occurrence(d(2026, 11, 30), 1), Some(d(2027, 2, 28)));
        assert_eq!(Period::Custom { every: 1, unit: Unit::Month }.occurrence(d(2024, 1, 31), 1), Some(d(2024, 2, 29)));
    }

    #[test]
    fn next_date_is_today_when_due_today_and_future_anchor_is_kept() {
        let today = d(2026, 11, 25);
        assert_eq!(Period::Monthly.next_on_or_after(d(2026, 10, 25), today), today);
        assert_eq!(Period::Monthly.next_on_or_after(d(2026, 10, 26), today), d(2026, 11, 26));
        assert_eq!(Period::Monthly.next_on_or_after(d(2027, 3, 3), today), d(2027, 3, 3));
        assert_eq!(Period::Weekly.next_on_or_after(d(2026, 11, 1), today), d(2026, 11, 29));
    }

    #[test]
    fn long_gaps_roll_forward() {
        let anchor = d(2020, 5, 15);
        let today = d(2026, 11, 25);
        assert_eq!(Period::Monthly.next_on_or_after(anchor, today), d(2026, 12, 15));
        assert_eq!(Period::Yearly.next_on_or_after(anchor, today), d(2027, 5, 15));
        assert_eq!(Period::Quarterly.next_on_or_after(anchor, today), d(2027, 2, 15));
        let c = Period::Custom { every: 10, unit: Unit::Day };
        let n = c.next_on_or_after(anchor, today);
        assert!(n >= today && (n - today).num_days() < 10);
        assert_eq!((n - anchor).num_days() % 10, 0);
    }

    #[test]
    fn dates_between_include_every_payment() {
        let a = d(2026, 11, 1);
        let w = Period::Weekly.between(a, d(2026, 11, 25), d(2026, 12, 25));
        assert_eq!(w, vec![d(2026, 11, 29), d(2026, 12, 6), d(2026, 12, 13), d(2026, 12, 20)]);
        let m = Period::Monthly.between(d(2026, 1, 31), d(2026, 2, 1), d(2026, 4, 30));
        assert_eq!(m, vec![d(2026, 2, 28), d(2026, 3, 31), d(2026, 4, 30)]);
        assert!(Period::Yearly.between(a, d(2026, 11, 25), d(2026, 12, 25)).is_empty());
    }

    #[test]
    fn custom_values_are_clamped() {
        assert_eq!(Period::Custom { every: 0, unit: Unit::Day }.sanitized(), Period::Custom { every: 1, unit: Unit::Day });
        assert_eq!(Period::of_kind(Kind::Custom, 5000, Unit::Week), Period::Custom { every: 999, unit: Unit::Week });
        // a zero from a hand-edited file must not divide by zero
        let _ = Period::Custom { every: 0, unit: Unit::Day }.per_year();
    }

    #[test]
    fn serde_round_trip() {
        for p in [Period::Weekly, Period::Monthly, Period::Quarterly, Period::Yearly, Period::Custom { every: 3, unit: Unit::Week }] {
            let s = serde_json::to_string(&p).unwrap();
            assert_eq!(serde_json::from_str::<Period>(&s).unwrap(), p);
        }
    }
}
