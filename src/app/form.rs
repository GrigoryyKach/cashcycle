//! The add/edit form: its state, validation and the conversion into a subscription.

use crate::fmt;
use crate::i18n::Lang;
use crate::model::money;
use crate::model::period::{Kind, Period, Unit, MAX_EVERY};
use crate::model::types::{normalize_url, Subscription, MAX_NAME, MAX_NOTE};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Clone)]
pub struct Form {
    /// `None` for a new subscription.
    pub id: Option<String>,
    pub name: String,
    pub url: String,
    pub price: String,
    pub currency: String,
    pub kind: Kind,
    pub every: String,
    pub unit: Unit,
    pub date: String,
    pub category: String,
    pub adding_category: bool,
    pub new_category: String,
    pub note: String,
    /// Set once Save was pressed with mistakes: errors are shown only after that.
    pub tried: bool,
    pub focus: bool,
}

/// Which fields are wrong.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Errors {
    pub name: bool,
    pub url: bool,
    pub price: bool,
    pub date: bool,
    pub every: bool,
}

impl Errors {
    pub fn any(&self) -> bool {
        self.name || self.url || self.price || self.date || self.every
    }
}

/// A validated form.
pub struct Built {
    pub name: String,
    pub url: String,
    pub price: Decimal,
    pub currency: String,
    pub period: Period,
    pub start: NaiveDate,
    pub category: String,
    pub note: String,
    /// A category typed into "New category", to be added to the list.
    pub new_category: Option<String>,
}

impl Form {
    pub fn new(currency: &str, lang: Lang, today: NaiveDate) -> Form {
        Form {
            id: None,
            name: String::new(),
            url: String::new(),
            price: String::new(),
            currency: currency.to_string(),
            kind: Kind::Monthly,
            every: "2".into(),
            unit: Unit::Week,
            date: fmt::date_input(lang, today),
            category: String::new(),
            adding_category: false,
            new_category: String::new(),
            note: String::new(),
            tried: false,
            focus: true,
        }
    }

    pub fn from_sub(s: &Subscription, lang: Lang) -> Form {
        let (every, unit) = match s.period {
            Period::Custom { every, unit } => (every.to_string(), unit),
            _ => ("2".to_string(), Unit::Week),
        };
        Form {
            id: Some(s.id.clone()),
            name: s.name.clone(),
            url: s.url.clone(),
            price: money::edit_text(s.price, lang != Lang::En),
            currency: s.currency.clone(),
            kind: s.period.kind(),
            every,
            unit,
            date: fmt::date_input(lang, s.start),
            category: s.category.clone(),
            adding_category: false,
            new_category: String::new(),
            note: s.note.clone(),
            tried: false,
            focus: true,
        }
    }

    fn every_value(&self) -> Option<u32> {
        self.every.trim().parse::<u32>().ok().filter(|n| (1..=MAX_EVERY).contains(n))
    }

    pub fn errors(&self) -> Errors {
        Errors {
            name: self.name.trim().is_empty(),
            url: normalize_url(&self.url).is_err(),
            price: money::parse_price(&self.price).is_none(),
            date: fmt::parse_date(&self.date).is_none(),
            every: self.kind == Kind::Custom && self.every_value().is_none(),
        }
    }

    pub fn build(&self) -> Result<Built, Errors> {
        let e = self.errors();
        if e.any() {
            return Err(e);
        }
        let new_category = if self.adding_category && !self.new_category.trim().is_empty() { Some(self.new_category.trim().to_string()) } else { None };
        Ok(Built {
            name: self.name.trim().chars().take(MAX_NAME).collect(),
            url: normalize_url(&self.url).unwrap_or_default(),
            price: money::parse_price(&self.price).unwrap_or_default(),
            currency: self.currency.clone(),
            period: Period::of_kind(self.kind, self.every_value().unwrap_or(1), self.unit),
            start: fmt::parse_date(&self.date).unwrap_or_default(),
            category: self.category.clone(),
            note: self.note.trim().chars().take(MAX_NOTE).collect(),
            new_category,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()
    }

    fn filled() -> Form {
        let mut f = Form::new("USD", Lang::En, today());
        f.name = "  Netflix ".into();
        f.price = "9,99".into();
        f.url = "netflix.com".into();
        f
    }

    #[test]
    fn a_complete_form_builds() {
        let b = filled().build().ok().unwrap();
        assert_eq!(b.name, "Netflix");
        assert_eq!(b.url, "https://netflix.com");
        assert_eq!(b.price, Decimal::from_str("9.99").unwrap());
        assert_eq!(b.period, Period::Monthly);
        assert_eq!(b.start, today());
    }

    #[test]
    fn mistakes_are_reported_per_field() {
        let mut f = Form::new("USD", Lang::En, today());
        let e = f.build().err().unwrap();
        assert!(e.name && e.price && !e.url && !e.date && !e.every);
        f.name = "x".into();
        f.price = "abc".into();
        f.url = "not a url".into();
        f.date = "31.02.2026".into();
        let e = f.build().err().unwrap();
        assert_eq!(e, Errors { name: false, url: true, price: true, date: true, every: false });
        f.kind = Kind::Custom;
        f.every = "0".into();
        assert!(f.errors().every);
        f.every = "1000".into();
        assert!(f.errors().every);
        f.every = "10".into();
        assert!(!f.errors().every);
    }

    #[test]
    fn custom_period_is_built_and_every_is_ignored_for_fixed_ones() {
        let mut f = filled();
        f.kind = Kind::Custom;
        f.every = "3".into();
        f.unit = Unit::Month;
        assert_eq!(f.build().ok().unwrap().period, Period::Custom { every: 3, unit: Unit::Month });
        f.kind = Kind::Yearly;
        f.every = "garbage".into();
        assert_eq!(f.build().ok().unwrap().period, Period::Yearly);
    }

    #[test]
    fn editing_round_trips_the_subscription() {
        let mut f = filled();
        f.kind = Kind::Custom;
        f.every = "6".into();
        f.unit = Unit::Week;
        f.note = "family".into();
        let b = f.build().ok().unwrap();
        let sub = Subscription {
            id: "1".into(),
            name: b.name,
            url: b.url,
            price: b.price,
            currency: b.currency,
            period: b.period,
            start: b.start,
            category: b.category,
            note: b.note,
            paused: None,
        };
        for lang in Lang::ALL {
            let g = Form::from_sub(&sub, lang);
            let b2 = g.build().ok().unwrap();
            assert_eq!((b2.price, b2.period, b2.start), (sub.price, sub.period, sub.start));
        }
    }

    #[test]
    fn a_typed_new_category_is_passed_on() {
        let mut f = filled();
        f.adding_category = true;
        f.new_category = " Books ".into();
        assert_eq!(f.build().ok().unwrap().new_category.as_deref(), Some("Books"));
        f.adding_category = false;
        assert!(f.build().ok().unwrap().new_category.is_none());
    }
}
