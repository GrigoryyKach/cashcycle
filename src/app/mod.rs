//! The application: state, the per-frame logic and the page layout. Views live in the sibling modules.

use std::path::PathBuf;
use std::time::Duration;

use chrono::{Local, NaiveDate};
use eframe::egui::{self, Margin, Ui};
use rust_decimal::Decimal;

use crate::config::{Config, Window};
use crate::i18n::{self, Lang};
use crate::model::calc;
use crate::model::rates::Rates;
use crate::model::storage::{self, LoadOutcome};
use crate::model::types::{new_id, Data, Subscription};
use crate::model::{money, rates};
use crate::nimvoro::Updater;
use crate::ratesvc::{self, RatesSvc};
use crate::shot::Shot;
use crate::strings::{self, S};
use crate::theme::{self, Pal};

mod datepick;
mod dialogs;
pub mod form;
mod main_view;
mod settings_view;
pub mod widgets;

pub use form::Form;

/// The app accent: lime. The light shade is dark enough to stay readable as text on the light surfaces.
pub const ACCENT: theme::Accent = theme::Accent::new(0xa3d14a, 0x4d7c0f, 0x141415, 0xffffff);
/// Minimum row height, so labels, buttons and icons that share a row sit on one line.
pub const ROW_H: f32 = 34.0;

pub enum Dialog {
    Edit(Form),
    Delete(String),
    Pause { id: String, until_on: bool, until: String },
    Resume { id: String, date: String },
    Corrupt(PathBuf),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Main,
    Settings,
}

/// Command-line options (also used to drive the screenshots).
#[derive(Default)]
pub struct Opts {
    pub demo: bool,
    pub shot: Option<Shot>,
    pub view: String,
    pub theme: Option<theme::ThemeMode>,
    pub lang: Option<Lang>,
    pub offline: bool,
}

impl Opts {
    pub fn parse(args: &[String]) -> Opts {
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
        Opts {
            demo: args.iter().any(|a| a == "--demo"),
            shot: Shot::from_args(args),
            view: val("--view").unwrap_or_default(),
            theme: val("--theme").and_then(|t| match t.as_str() {
                "dark" => Some(theme::ThemeMode::Dark),
                "light" => Some(theme::ThemeMode::Light),
                _ => None,
            }),
            lang: val("--lang").and_then(|l| Lang::from_code(&l)),
            offline: args.iter().any(|a| a == "--offline"),
        }
    }
}

pub struct App {
    pub cfg: Config,
    pub lang: Lang,
    pub dir: PathBuf,
    pub data: Data,
    pub updater: Updater,
    pub rates: RatesSvc,
    /// Snapshot of the rates for this frame.
    pub rv: ratesvc::View,
    pub today: NaiveDate,
    pub view: View,
    pub search: String,
    pub dialog: Option<Dialog>,
    pub generation: u64,
    pub demo: bool,
    data_dirty: Option<f64>,
    cfg_dirty: Option<f64>,
    shot: Option<Shot>,
    last_window: Option<Window>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, mut cfg: Config, dir: PathBuf, mut opts: Opts) -> Self {
        theme::set_accent(ACCENT);
        if let Some(t) = opts.theme {
            cfg.theme = t;
        }
        theme::install(&cc.egui_ctx, cfg.theme);
        cc.egui_ctx.all_styles_mut(|s| s.spacing.interact_size.y = ROW_H);
        let lang = opts.lang.unwrap_or_else(|| i18n::resolve(cfg.language.as_deref()));
        let mut cfg_dirty = cfg.resolve(lang).then_some(0.0);
        let now = Local::now().naive_local();
        let today = now.date();

        let (mut data, outcome, initial_rates) = if opts.demo {
            (crate::demo::data(today), LoadOutcome::Fresh, Some(crate::demo::rates()))
        } else {
            let (d, o) = storage::load(&dir, &now.format("%Y%m%d-%H%M%S").to_string());
            (d, o, rates::load_cache(&dir))
        };
        let mut data_dirty = None;
        if data.auto_resume(today) > 0 {
            data_dirty = Some(0.0);
        }
        if opts.demo {
            cfg_dirty = None;
            data_dirty = None;
        }
        // The disk cache is only the fallback; the service downloads fresh rates on start.
        let rsvc = RatesSvc::new(dir.clone(), initial_rates, opts.demo);
        if opts.demo && opts.offline {
            rsvc.set_failed(true);
        }
        if opts.view == "norates" {
            rsvc.set_failed(true);
        }

        let updater = Updater::new("cashcycle", crate::REPO_URL.trim_start_matches("https://github.com/"), env!("CARGO_PKG_VERSION"));
        if cfg.check_updates && !opts.demo {
            let ctx = cc.egui_ctx.clone();
            updater.check(move || ctx.request_repaint());
        }

        let mut app = Self {
            lang,
            updater,
            rates: rsvc,
            rv: ratesvc::View::default(),
            today,
            view: View::Main,
            search: String::new(),
            dialog: None,
            generation: 0,
            demo: opts.demo,
            data_dirty,
            cfg_dirty,
            shot: opts.shot.take(),
            last_window: None,
            cfg,
            dir,
            data,
        };
        if let LoadOutcome::Recovered { moved_to } = outcome {
            app.open(Dialog::Corrupt(moved_to));
        }
        app.apply_view(&opts.view);
        if opts.view == "norates" {
            app.rv = app.rates.view();
        }
        app
    }

    /// Screenshot helper: jump straight to a page or dialog.
    fn apply_view(&mut self, name: &str) {
        let first = self.data.subs.first().map(|s| s.id.clone());
        match name {
            "settings" => self.view = View::Settings,
            "new" => self.open(Dialog::Edit(Form::new(&self.cfg.default_currency, self.lang, self.today))),
            "edit" => {
                if let Some(s) = first.and_then(|id| self.data.sub(&id).cloned()) {
                    self.open(Dialog::Edit(Form::from_sub(&s, self.lang)));
                }
            }
            "delete" => {
                if let Some(id) = first {
                    self.open(Dialog::Delete(id));
                }
            }
            "pause" => {
                if let Some(id) = first {
                    self.open(Dialog::Pause { id, until_on: false, until: String::new() });
                }
            }
            "empty" => self.data = Data::default(),
            _ => {}
        }
    }

    pub fn open(&mut self, d: Dialog) {
        if self.dialog.is_none() {
            self.generation += 1;
        }
        self.dialog = Some(d);
    }

    pub fn s(&self) -> &'static S {
        strings::TABLE.get(self.lang)
    }

    /// Russian and Ukrainian write the decimal comma.
    pub fn comma(&self) -> bool {
        self.lang != Lang::En
    }

    pub fn money(&self, d: Decimal, code: &str) -> String {
        money::format_money(d, code, self.comma())
    }

    pub fn rates_ref(&self) -> Option<&Rates> {
        self.rv.rates.as_ref()
    }

    pub fn total_currency(&self) -> &str {
        &self.cfg.total_currency
    }

    pub fn touch_cfg(&mut self, ctx: &egui::Context) {
        self.cfg_dirty = Some(ctx.input(|i| i.time));
    }

    pub fn touch_data(&mut self, ctx: &egui::Context) {
        self.data_dirty = Some(ctx.input(|i| i.time));
    }

    /// Writes pending changes now. Returns `false` if something could not be written.
    pub fn flush(&mut self) -> bool {
        if self.demo {
            self.data_dirty = None;
            self.cfg_dirty = None;
            return true;
        }
        let mut ok = true;
        if self.data_dirty.take().is_some() {
            ok &= storage::save(&self.dir, &self.data, self.today).is_ok();
        }
        if self.cfg_dirty.take().is_some() {
            ok &= self.cfg.save_to(&self.dir);
        }
        ok
    }

    /// Adds or updates a subscription from a validated form.
    pub fn save_form(&mut self, ctx: &egui::Context, form: &Form, built: form::Built) {
        let category = match &built.new_category {
            Some(c) => self.data.add_category(c),
            None => built.category.clone(),
        };
        let paused = form.id.as_ref().and_then(|id| self.data.sub(id)).and_then(|s| s.paused);
        let sub = Subscription {
            id: form.id.clone().unwrap_or_else(new_id),
            name: built.name,
            url: built.url,
            price: built.price,
            currency: built.currency.clone(),
            period: built.period,
            start: built.start,
            category,
            note: built.note,
            paused,
        };
        match self.data.sub_mut(&sub.id) {
            Some(slot) => *slot = sub,
            None => self.data.subs.push(sub),
        }
        self.cfg.note_currency(&built.currency);
        self.touch_cfg(ctx);
        self.touch_data(ctx);
    }

    pub fn delete_sub(&mut self, ctx: &egui::Context, id: &str) {
        self.data.subs.retain(|s| s.id != id);
        self.touch_data(ctx);
    }

    fn track_window(&mut self, ctx: &egui::Context) {
        let (inner, outer) = ctx.input(|i| (i.viewport().inner_rect, i.viewport().outer_rect));
        let (Some(inner), Some(outer)) = (inner, outer) else { return };
        if ctx.input(|i| i.viewport().minimized.unwrap_or(false) || i.viewport().maximized.unwrap_or(false)) {
            return;
        }
        let w = Window { w: inner.width(), h: inner.height(), x: Some(outer.min.x), y: Some(outer.min.y) };
        let changed = match self.last_window {
            Some(l) => (l.w - w.w).abs() > 1.0 || (l.h - w.h).abs() > 1.0 || l.x != w.x || l.y != w.y,
            None => false,
        };
        if self.last_window.is_none() || changed {
            if changed {
                self.cfg.window = Some(w);
                self.touch_cfg(ctx);
            }
            self.last_window = Some(w);
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.updater.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if self.updater.should_exit() {
            self.flush();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.today = Local::now().date_naive();
        // A pause with an end date resumes by itself.
        if self.data.auto_resume(self.today) > 0 {
            self.touch_data(ctx);
        }
        self.rates.keep_fresh(ctx);
        self.rv = self.rates.view();
        if self.rv.fetching {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        let t = ctx.input(|i| i.time);
        if self.data_dirty.is_some_and(|d| t - d > 0.4) || self.cfg_dirty.is_some_and(|d| t - d > 0.4) {
            self.flush();
        }
        if self.data_dirty.is_some() || self.cfg_dirty.is_some() {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        if self.shot.is_none() && !self.demo {
            self.track_window(ctx);
        }
        // Keep "today", the countdowns and the rates age fresh without a busy loop.
        ctx.request_repaint_after(Duration::from_secs(30));
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let p = Pal::of(ui.visuals().dark_mode);
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, p.bg);
        let inner = egui::Rect::from_min_max(rect.min + egui::vec2(28.0, 20.0), rect.max - egui::vec2(28.0, 0.0));
        let mut page = ui.new_child(egui::UiBuilder::new().id_salt("page").max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
        page.set_clip_rect(rect);
        match self.view {
            View::Main => self.main_view(&mut page, &p),
            View::Settings => self.settings_view(&mut page, &p),
        }
        self.dialog_ui(&ctx, &p);
        if let Some(shot) = &mut self.shot {
            shot.tick(&ctx);
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.flush();
    }
}

/// Inner margin used by the page cards.
pub fn card_margin() -> Margin {
    Margin::symmetric(16, 14)
}

/// The totals of the current data in the card currency (shared by the views).
pub fn current_totals(app: &App) -> calc::Totals {
    calc::totals(&app.data, app.total_currency(), app.rates_ref())
}

#[cfg(test)]
mod accent_tests {
    use super::ACCENT;
    use crate::theme::{self, Pal};

    #[test]
    fn accent_is_readable_in_both_themes() {
        theme::set_accent(ACCENT);
        assert!(theme::accent_problems().is_empty(), "{:?}", theme::accent_problems());
        let l = Pal::light();
        for (name, s) in [("bg", l.bg), ("card", l.card), ("raised", l.raised)] {
            assert!(theme::contrast(l.accent, s) >= 4.5, "light accent on {name}");
        }
    }
}
