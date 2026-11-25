//! Localised date and time text. Names come from the app's own tables, not from the OS.

use crate::i18n::Lang;
use chrono::{Datelike, NaiveDate, Weekday};

const MONTHS_EN: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const MONTHS_RU: [&str; 12] =
    ["Январь", "Февраль", "Март", "Апрель", "Май", "Июнь", "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь"];
const MONTHS_UK: [&str; 12] =
    ["Січень", "Лютий", "Березень", "Квітень", "Травень", "Червень", "Липень", "Серпень", "Вересень", "Жовтень", "Листопад", "Грудень"];
const MONTHS_RU_GEN: [&str; 12] =
    ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"];
const MONTHS_UK_GEN: [&str; 12] =
    ["січня", "лютого", "березня", "квітня", "травня", "червня", "липня", "серпня", "вересня", "жовтня", "листопада", "грудня"];
const MONTHS_SHORT_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const MONTHS_SHORT_RU: [&str; 12] = ["янв", "фев", "мар", "апр", "мая", "июн", "июл", "авг", "сен", "окт", "ноя", "дек"];
const MONTHS_SHORT_UK: [&str; 12] = ["січ", "лют", "бер", "кві", "тра", "чер", "лип", "сер", "вер", "жов", "лис", "гру"];

const DAYS_EN: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const DAYS_RU: [&str; 7] = ["понедельник", "вторник", "среда", "четверг", "пятница", "суббота", "воскресенье"];
const DAYS_UK: [&str; 7] = ["понеділок", "вівторок", "середа", "четвер", "пʼятниця", "субота", "неділя"];
const DAYS_SHORT_EN: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const DAYS_SHORT_RU: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];
const DAYS_SHORT_UK: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Нд"];

fn pick<'a>(lang: Lang, en: &'a [&'a str], ru: &'a [&'a str], uk: &'a [&'a str], i: usize) -> &'a str {
    match lang {
        Lang::En => en[i],
        Lang::Ru => ru[i],
        Lang::Uk => uk[i],
    }
}

/// "November", "Ноябрь", "Листопад".
pub fn month_name(lang: Lang, month: u32) -> &'static str {
    pick(lang, &MONTHS_EN, &MONTHS_RU, &MONTHS_UK, (month as usize + 11) % 12)
}

pub fn month_title(lang: Lang, year: i32, month: u32) -> String {
    format!("{} {}", month_name(lang, month), year)
}

pub fn weekday_short(lang: Lang, w: Weekday) -> &'static str {
    pick(lang, &DAYS_SHORT_EN, &DAYS_SHORT_RU, &DAYS_SHORT_UK, w.num_days_from_monday() as usize)
}

pub fn weekday_long(lang: Lang, w: Weekday) -> &'static str {
    pick(lang, &DAYS_EN, &DAYS_RU, &DAYS_UK, w.num_days_from_monday() as usize)
}

/// "Nov 17" / "17 ноя" / "17 лис".
pub fn date_short(lang: Lang, d: NaiveDate) -> String {
    let i = d.month0() as usize;
    match lang {
        Lang::En => format!("{} {}", MONTHS_SHORT_EN[i], d.day()),
        Lang::Ru => format!("{} {}", d.day(), MONTHS_SHORT_RU[i]),
        Lang::Uk => format!("{} {}", d.day(), MONTHS_SHORT_UK[i]),
    }
}

/// "Nov 17, 2026" / "17 ноя 2026".
pub fn date_full(lang: Lang, d: NaiveDate) -> String {
    format!("{} {}", date_short(lang, d), d.year())
}

/// "Tuesday, November 17" / "вторник, 17 ноября".
pub fn date_long(lang: Lang, d: NaiveDate) -> String {
    let i = d.month0() as usize;
    let wd = weekday_long(lang, d.weekday());
    match lang {
        Lang::En => format!("{wd}, {} {}", MONTHS_EN[i], d.day()),
        Lang::Ru => format!("{wd}, {} {}", d.day(), MONTHS_RU_GEN[i]),
        Lang::Uk => format!("{wd}, {} {}", d.day(), MONTHS_UK_GEN[i]),
    }
}


/// "Nov 25, 14:00" / "25 ноя, 14:00": a moment in the local time zone.
pub fn moment(lang: Lang, unix: i64) -> String {
    use chrono::TimeZone;
    match chrono::Local.timestamp_opt(unix, 0).single() {
        Some(t) => format!("{}, {:02}:{:02}", date_short(lang, t.date_naive()), chrono::Timelike::hour(&t), chrono::Timelike::minute(&t)),
        None => String::new(),
    }
}

/// Accepts `2026-12-31`, `31.12.2026`, `31/12/2026`, `31-12-2026`. Two-digit years become 20xx.
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(['-', '.', '/']).map(str::trim).collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let n: Vec<u32> = parts.iter().map(|p| p.parse().ok()).collect::<Option<_>>()?;
    let (y, m, d) = if parts[0].len() == 4 { (n[0], n[1], n[2]) } else { (if parts[2].len() == 2 { 2000 + n[2] } else { n[2] }, n[1], n[0]) };
    if !(1900..=2200).contains(&y) {
        return None;
    }
    NaiveDate::from_ymd_opt(y as i32, m, d)
}

/// The text a date field starts with: `2026-12-31` in English, `31.12.2026` in Russian and Ukrainian.
pub fn date_input(lang: Lang, d: NaiveDate) -> String {
    match lang {
        Lang::En => d.format("%Y-%m-%d").to_string(),
        _ => d.format("%d.%m.%Y").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn names() {
        assert_eq!(month_title(Lang::Uk, 2026, 11), "Листопад 2026");
        assert_eq!(date_long(Lang::Ru, d(2026, 11, 17)), "вторник, 17 ноября");
        assert_eq!(date_short(Lang::En, d(2026, 11, 17)), "Nov 17");
        assert_eq!(date_full(Lang::Uk, d(2026, 11, 17)), "17 лис 2026");
    }

    #[test]
    fn date_input_formats() {
        let want = Some(d(2026, 12, 31));
        for s in ["2026-12-31", "31.12.2026", "31/12/2026", " 31-12-2026 ", "31.12.26"] {
            assert_eq!(parse_date(s), want, "{s}");
        }
        for bad in ["", "2026-13-01", "31.02.2026", "abc", "1.2", "2026-12-31-1", "0000-01-01"] {
            assert_eq!(parse_date(bad), None, "{bad}");
        }
        assert_eq!(date_input(Lang::En, d(2026, 1, 5)), "2026-01-05");
        assert_eq!(date_input(Lang::Ru, d(2026, 1, 5)), "05.01.2026");
        assert_eq!(parse_date(&date_input(Lang::Uk, d(2026, 1, 5))), Some(d(2026, 1, 5)));
    }
}
