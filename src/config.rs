//! Settings (`config.json`) and the data folder.

use crate::i18n::Lang;
use crate::model::calc::Sort;
use crate::model::storage::write_atomic;
use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `%APPDATA%\Cashcycle` (`~/.config/Cashcycle` on Linux), or the folder from `CASHCYCLE_DATA_DIR`
/// (used for tests and screenshots).
pub fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("CASHCYCLE_DATA_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(d);
    }
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("Cashcycle")
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum FirstDay {
    #[default]
    Monday,
    Sunday,
    Saturday,
}

impl FirstDay {
    pub fn weekday(self) -> chrono::Weekday {
        match self {
            FirstDay::Monday => chrono::Weekday::Mon,
            FirstDay::Sunday => chrono::Weekday::Sun,
            FirstDay::Saturday => chrono::Weekday::Sat,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Window {
    pub w: f32,
    pub h: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    /// Interface language code (`en`, `ru`, `uk`); `None` until first resolved.
    pub language: Option<String>,
    pub theme: ThemeMode,
    /// Currency a new subscription starts with. Empty until first resolved.
    pub default_currency: String,
    /// Currency of the total card. Empty until first resolved.
    pub total_currency: String,
    pub first_day: FirstDay,
    /// Currencies picked most recently, newest first (shown at the top of the pickers).
    pub recent_currencies: Vec<String>,
    pub sort: Sort,
    pub check_updates: bool,
    pub window: Option<Window>,
    #[serde(skip)]
    pub fresh: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: None,
            theme: ThemeMode::System,
            default_currency: String::new(),
            total_currency: String::new(),
            first_day: FirstDay::Monday,
            recent_currencies: Vec::new(),
            sort: Sort::Next,
            check_updates: true,
            window: None,
            fresh: false,
        }
    }
}

/// UAH for Ukrainian and Russian interfaces (the first run), USD otherwise.
pub fn currency_for(lang: Lang) -> &'static str {
    match lang {
        Lang::Uk | Lang::Ru => "UAH",
        Lang::En => "USD",
    }
}

impl Config {
    pub fn load_from(dir: &std::path::Path) -> Self {
        let text = std::fs::read_to_string(dir.join("config.json")).ok();
        let mut cfg: Config = text.as_deref().and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok()).unwrap_or_default();
        cfg.fresh = text.is_none();
        cfg
    }

    /// Remembers a picked currency (at most 3, newest first).
    pub fn note_currency(&mut self, code: &str) {
        let code = code.to_uppercase();
        self.recent_currencies.retain(|c| *c != code);
        self.recent_currencies.insert(0, code);
        self.recent_currencies.truncate(3);
    }

    /// Fills what is still empty from the interface language. Returns `true` when something changed.
    pub fn resolve(&mut self, lang: Lang) -> bool {
        let mut changed = false;
        if self.language.as_deref() != Some(lang.code()) {
            self.language = Some(lang.code().to_string());
            changed = true;
        }
        for cur in [&mut self.default_currency, &mut self.total_currency] {
            let ok = crate::model::currency::find(cur).is_some();
            if !ok {
                *cur = currency_for(lang).to_string();
                changed = true;
            } else if cur.as_str() != cur.to_uppercase() {
                *cur = cur.to_uppercase();
                changed = true;
            }
        }
        let before = self.recent_currencies.len();
        self.recent_currencies.retain(|c| crate::model::currency::find(c).is_some());
        changed |= before != self.recent_currencies.len();
        changed
    }

    /// Writes the settings. `false` if they could not be written.
    pub fn save_to(&self, dir: &std::path::Path) -> bool {
        match serde_json::to_vec_pretty(self) {
            Ok(json) => write_atomic(&dir.join("config.json"), &json).is_ok(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("cashcycle-cfg-{name}-{}", crate::model::types::new_id()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn missing_config_is_fresh_with_defaults() {
        let dir = tmp("fresh");
        let c = Config::load_from(&dir);
        assert!(c.fresh);
        assert_eq!(c.theme, ThemeMode::System);
        assert_eq!(c.first_day, FirstDay::Monday);
        assert!(c.check_updates);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tmp("rt");
        let mut c = Config::default();
        c.language = Some("uk".into());
        c.theme = ThemeMode::Dark;
        c.default_currency = "EUR".into();
        c.total_currency = "UAH".into();
        c.first_day = FirstDay::Sunday;
        c.sort = Sort::Price;
        assert!(c.save_to(&dir));
        let l = Config::load_from(&dir);
        assert!(!l.fresh);
        assert_eq!(l.language.as_deref(), Some("uk"));
        assert_eq!(l.theme, ThemeMode::Dark);
        assert_eq!((l.default_currency.as_str(), l.total_currency.as_str()), ("EUR", "UAH"));
        assert_eq!(l.first_day, FirstDay::Sunday);
        assert_eq!(l.sort, Sort::Price);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn bom_unknown_fields_and_garbage_are_tolerated() {
        let dir = tmp("bom");
        std::fs::write(dir.join("config.json"), "\u{feff}{\"theme\":\"light\",\"future_key\":1}").unwrap();
        assert_eq!(Config::load_from(&dir).theme, ThemeMode::Light);
        std::fs::write(dir.join("config.json"), "{ nope").unwrap();
        let c = Config::load_from(&dir);
        assert_eq!(c.theme, ThemeMode::System);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn currencies_start_from_the_language_and_bad_codes_are_replaced() {
        let mut c = Config::default();
        assert!(c.resolve(Lang::Uk));
        assert_eq!((c.default_currency.as_str(), c.total_currency.as_str()), ("UAH", "UAH"));
        assert!(!c.resolve(Lang::Uk));
        let mut e = Config::default();
        e.total_currency = "eur".into();
        e.default_currency = "nonsense".into();
        e.resolve(Lang::En);
        assert_eq!((e.default_currency.as_str(), e.total_currency.as_str()), ("USD", "EUR"));
    }

    #[test]
    fn recent_currencies_are_newest_first_and_never_excluded() {
        let mut c = Config::default();
        for x in ["eur", "PLN", "usd", "PLN"] {
            c.note_currency(x);
        }
        assert_eq!(c.recent_currencies, vec!["PLN", "USD", "EUR"]);
        c.recent_currencies = vec!["RUB".into(), "GBP".into()];
        c.default_currency = "RUB".into();
        c.resolve(Lang::En);
        assert_eq!(c.recent_currencies, vec!["GBP"]);
        assert_eq!(c.default_currency, "USD");
    }

    #[test]
    fn language_order_config_then_marker_then_english() {
        assert_eq!(i18n::resolve(Some("ru")), Lang::Ru);
        assert_eq!(i18n::resolve(Some("uk")), Lang::Uk);
        if i18n::marker_language().is_none() {
            assert_eq!(i18n::resolve(None), Lang::En);
            assert_eq!(i18n::resolve(Some("klingon")), Lang::En);
        }
    }
}
