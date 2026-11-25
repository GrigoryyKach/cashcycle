//! Modal dialogs: add/edit, delete confirmation, pause, resume and the "data file was unreadable" notice.

use super::datepick;
use super::form::Form;
use super::widgets::{self, Icon};
use super::{App, Dialog};
use crate::fmt;
use crate::model::period::{Kind, Unit};
use crate::model::types::normalize_url;
use crate::strings;
use crate::theme::{self, Pal};
use eframe::egui::{self, Key, RichText, ScrollArea, Ui};

enum Act {
    None,
    Close,
    Save,
    AskDiscard,
    BackToEdit,
    Delete(String),
    Pause(String),
    Resume(String),
}

impl App {
    pub(super) fn dialog_ui(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some(mut dialog) = self.dialog.take() else { return };
        let t = self.s();
        let lang = self.lang;
        let gen = self.generation;
        // A popup (currency list, date picker) inside the dialog gets the first click outside and the first Escape.
        let popup = egui::Popup::is_any_open(ctx);
        let esc = ctx.input(|i| i.key_pressed(Key::Escape)) && !popup;
        let enter = ctx.input(|i| i.key_pressed(Key::Enter)) && !popup;
        let mut act = Act::None;
        match &mut dialog {
            Dialog::Edit(form) => {
                let cats = self.data.all_categories();
                let (first, today, recent) = (self.cfg.first_day.weekday(), self.today, self.cfg.recent_currencies.clone());
                let max_h = (ctx.content_rect().height() - 90.0).max(300.0);
                let backdrop = widgets::modal(ctx, p, "edit", gen, 500.0, |ui| {
                    ScrollArea::vertical().max_height(max_h).auto_shrink([true, true]).show(ui, |ui| {
                        ui.set_width(500.0 - 12.0);
                        edit_body(ui, p, form, lang, &cats, &recent, first, today, self.data.categories.is_empty());
                    });
                    ui.add_space(14.0);
                    let editing = form.id.clone();
                    let paused = editing.as_ref().and_then(|id| self.data.sub(id)).is_some_and(|s| s.is_paused());
                    let secondary: Vec<&str> = match &editing {
                        Some(_) => vec![if paused { t.resume } else { t.pause }, t.cancel],
                        None => vec![t.cancel],
                    };
                    let out = widgets::footer(ui, p, editing.as_ref().map(|_| t.delete), &secondary, if editing.is_some() { t.save } else { t.add }, true);
                    if out.primary {
                        act = Act::Save;
                    } else if out.left {
                        act = Act::Delete(editing.clone().unwrap_or_default());
                    } else if let Some(i) = out.secondary {
                        let is_cancel = i == secondary.len() - 1;
                        act = if is_cancel {
                            Act::Close
                        } else if paused {
                            Act::Resume(editing.clone().unwrap_or_default())
                        } else {
                            Act::Pause(editing.clone().unwrap_or_default())
                        };
                    }
                });
                // Escape and a click outside close a clean form at once and ask before throwing changes away.
                if (esc || (backdrop && !popup)) && matches!(act, Act::None) {
                    act = if form.dirty() { Act::AskDiscard } else { Act::Close };
                }
                if enter && matches!(act, Act::None) {
                    act = Act::Save;
                }
            }
            Dialog::Discard(_) => {
                let backdrop = widgets::modal(ctx, p, "discard", gen, 420.0, |ui| {
                    ui.label(RichText::new(t.discard_title).font(theme::medium(18.0)).color(p.text));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(RichText::new(t.discard_body).font(theme::reg(14.0)).color(p.text2)).wrap());
                    ui.add_space(20.0);
                    let out = widgets::destructive_footer(ui, p, t.keep_editing, t.discard);
                    if out.confirm {
                        act = Act::Close;
                    } else if out.cancel {
                        act = Act::BackToEdit;
                    }
                });
                if (esc || enter || (backdrop && !popup)) && matches!(act, Act::None) {
                    act = Act::BackToEdit;
                }
            }
            Dialog::Delete(id) => {
                let name = self.data.sub(id).map(|s| s.name.clone()).unwrap_or_default();
                let backdrop = widgets::modal(ctx, p, "delete", gen, 440.0, |ui| {
                    ui.label(RichText::new(t.delete_title.replace("{name}", &name)).font(theme::medium(18.0)).color(p.text));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(RichText::new(t.delete_body).font(theme::reg(14.0)).color(p.text2)).wrap());
                    ui.add_space(20.0);
                    let out = widgets::destructive_footer(ui, p, t.cancel, t.delete);
                    if out.confirm {
                        act = Act::Delete(id.clone());
                    } else if out.cancel {
                        act = Act::Close;
                    }
                });
                if esc || enter || (backdrop && !popup) {
                    act = Act::Close;
                }
            }
            Dialog::Pause { id, until_on, until } => {
                let name = self.data.sub(id).map(|s| s.name.clone()).unwrap_or_default();
                let (first, today) = (self.cfg.first_day.weekday(), self.today);
                let mut valid = true;
                let backdrop = widgets::modal(ctx, p, "pause", gen, 440.0, |ui| {
                    ui.label(RichText::new(t.pause_title.replace("{name}", &name)).font(theme::medium(18.0)).color(p.text));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(RichText::new(t.pause_body).font(theme::reg(14.0)).color(p.text2)).wrap());
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        theme::switch(ui, until_on, p);
                        ui.label(RichText::new(t.pause_until).font(theme::reg(14.0)).color(p.text));
                    });
                    if *until_on {
                        ui.add_space(8.0);
                        if until.is_empty() {
                            *until = fmt::date_input(lang, today + chrono::Days::new(30));
                        }
                        let (d, _) = datepick::date_field(ui, "pause-until", until, lang, first, today, 220.0, p);
                        valid = d.is_some_and(|d| d > today);
                    }
                    ui.add_space(20.0);
                    let out = widgets::footer(ui, p, None, &[t.cancel], t.pause, valid);
                    if out.primary {
                        act = Act::Pause(id.clone());
                    } else if out.secondary.is_some() {
                        act = Act::Close;
                    }
                });
                if esc || (backdrop && !popup) {
                    act = Act::Close;
                }
                if enter && valid && matches!(act, Act::None) {
                    act = Act::Pause(id.clone());
                }
            }
            Dialog::Resume { id, date } => {
                let name = self.data.sub(id).map(|s| s.name.clone()).unwrap_or_default();
                let (first, today) = (self.cfg.first_day.weekday(), self.today);
                let mut valid = false;
                let backdrop = widgets::modal(ctx, p, "resume", gen, 440.0, |ui| {
                    ui.label(RichText::new(t.resume_title.replace("{name}", &name)).font(theme::medium(18.0)).color(p.text));
                    ui.add_space(8.0);
                    ui.label(RichText::new(t.resume_body).font(theme::reg(14.0)).color(p.text2));
                    ui.add_space(10.0);
                    let (d, _) = datepick::date_field(ui, "resume-date", date, lang, first, today, 220.0, p);
                    valid = d.is_some();
                    ui.add_space(20.0);
                    let out = widgets::footer(ui, p, None, &[t.cancel], t.resume, valid);
                    if out.primary {
                        act = Act::Resume(id.clone());
                    } else if out.secondary.is_some() {
                        act = Act::Close;
                    }
                });
                if esc || (backdrop && !popup) {
                    act = Act::Close;
                }
                if enter && valid && matches!(act, Act::None) {
                    act = Act::Resume(id.clone());
                }
            }
            Dialog::Corrupt(path) => {
                let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                widgets::modal(ctx, p, "corrupt", gen, 460.0, |ui| {
                    ui.label(RichText::new(t.corrupt_title).font(theme::medium(18.0)).color(p.text));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(RichText::new(t.corrupt_body.replace("{file}", &file)).font(theme::reg(14.0)).color(p.text2)).wrap());
                    ui.add_space(20.0);
                    if widgets::footer(ui, p, None, &[], t.ok, true).primary {
                        act = Act::Close;
                    }
                });
                if esc || enter {
                    act = Act::Close;
                }
            }
        }

        let mut keep = true;
        match act {
            Act::None => {}
            Act::Close => keep = false,
            Act::AskDiscard => {
                if let Dialog::Edit(form) = &dialog {
                    dialog = Dialog::Discard(Box::new(form.clone()));
                }
            }
            Act::BackToEdit => {
                if let Dialog::Discard(form) = dialog {
                    dialog = Dialog::Edit(*form);
                }
            }
            Act::Save => {
                if let Dialog::Edit(form) = &mut dialog {
                    match form.build() {
                        Ok(built) => {
                            let f = form.clone();
                            self.save_form(ctx, &f, built);
                            keep = false;
                        }
                        Err(_) => form.tried = true,
                    }
                }
            }
            Act::Delete(id) => {
                // From the editor this asks first; from the confirmation it deletes.
                if matches!(dialog, Dialog::Edit(_)) {
                    dialog = Dialog::Delete(id);
                } else {
                    self.delete_sub(ctx, &id);
                    keep = false;
                }
            }
            Act::Pause(id) => {
                if let Dialog::Pause { until_on, until, .. } = &dialog {
                    let until = until_on.then(|| fmt::parse_date(until)).flatten();
                    self.data.pause(&id, self.today, until);
                    self.touch_data(ctx);
                    keep = false;
                } else {
                    dialog = Dialog::Pause { id, until_on: false, until: String::new() };
                }
            }
            Act::Resume(id) => {
                if let Dialog::Resume { date, .. } = &dialog {
                    let chosen = fmt::parse_date(date);
                    let natural = self.data.sub(&id).map(|s| s.next_payment(self.today));
                    // Keep the original schedule when the suggested day was accepted.
                    let anchor = if chosen == natural { None } else { chosen };
                    self.data.resume(&id, anchor);
                    self.touch_data(ctx);
                    keep = false;
                } else {
                    let next = self.data.sub(&id).map(|s| s.next_payment(self.today)).unwrap_or(self.today);
                    dialog = Dialog::Resume { id, date: fmt::date_input(lang, next) };
                }
            }
        }
        if keep {
            self.dialog = Some(dialog);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn edit_body(ui: &mut Ui, p: &Pal, form: &mut Form, lang: crate::i18n::Lang, cats: &[String], recent: &[String], first: chrono::Weekday, today: chrono::NaiveDate, _no_custom: bool) {
    let t = strings::TABLE.get(lang);
    ui.label(RichText::new(if form.id.is_some() { t.edit_sub } else { t.new_sub }).font(theme::medium(18.0)).color(p.text));
    ui.add_space(12.0);
    let err = if form.tried { form.errors() } else { Default::default() };
    let full = ui.available_width();

    // Name
    widgets::field(ui, t.f_name, p, |ui| {
        let r = widgets::input(ui, &mut form.name, t.f_name_hint, full);
        if form.focus {
            r.request_focus();
            form.focus = false;
        }
        if err.name {
            widgets::flag_error(ui, &r, p);
            ui.label(RichText::new(t.err_name).font(theme::reg(12.0)).color(p.negative));
        }
    });
    ui.add_space(10.0);

    // Link
    widgets::field(ui, t.f_link, p, |ui| {
        ui.horizontal(|ui| {
            let r = widgets::input(ui, &mut form.url, t.f_link_hint, full - 42.0);
            let valid = normalize_url(&form.url);
            if err.url {
                widgets::flag_error(ui, &r, p);
            }
            if let Ok(u) = &valid {
                if !u.is_empty() && widgets::icon_button(ui, Icon::Link, 34.0, p).on_hover_text(t.open_link).clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(u.clone()));
                }
            }
        });
        if err.url {
            ui.label(RichText::new(t.err_link).font(theme::reg(12.0)).color(p.negative));
        }
    });
    ui.add_space(10.0);

    // Price and currency
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.vertical(|ui| {
            widgets::field(ui, t.f_price, p, |ui| {
                let r = widgets::input(ui, &mut form.price, if lang == crate::i18n::Lang::En { "9.99" } else { "9,99" }, 140.0);
                if err.price {
                    widgets::flag_error(ui, &r, p);
                }
            });
        });
        ui.vertical(|ui| {
            widgets::field(ui, t.f_currency, p, |ui| {
                widgets::currency_combo(ui, "form-currency", &mut form.currency, recent, t.f_search_currency, 150.0, p);
            });
        });
    });
    if err.price {
        ui.label(RichText::new(t.err_price).font(theme::reg(12.0)).color(p.negative));
    }
    ui.add_space(10.0);

    // Billing period
    widgets::field(ui, t.f_billing, p, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let kinds: Vec<(Kind, String)> = Kind::ALL
                .iter()
                .map(|k| {
                    let name = match k {
                        Kind::Weekly => t.weekly,
                        Kind::Monthly => t.monthly,
                        Kind::Quarterly => t.quarterly,
                        Kind::Yearly => t.yearly,
                        Kind::Custom => t.custom,
                    };
                    (*k, name.to_string())
                })
                .collect();
            if let Some(k) = widgets::pick(ui, "form-kind", form.kind, &kinds, 170.0) {
                form.kind = k;
            }
            if form.kind == Kind::Custom {
                ui.label(RichText::new(t.f_every).font(theme::reg(13.0)).color(p.text2));
                let r = widgets::input(ui, &mut form.every, "2", 52.0);
                if err.every {
                    widgets::flag_error(ui, &r, p);
                }
                let units: Vec<(Unit, String)> = vec![(Unit::Day, t.unit_day.to_string()), (Unit::Week, t.unit_week.to_string()), (Unit::Month, t.unit_month.to_string())];
                if let Some(u) = widgets::pick(ui, "form-unit", form.unit, &units, 110.0) {
                    form.unit = u;
                }
            }
        });
        if err.every {
            ui.label(RichText::new(t.err_every).font(theme::reg(12.0)).color(p.negative));
        }
    });
    ui.add_space(10.0);

    // Date
    widgets::field(ui, t.f_date, p, |ui| {
        datepick::date_field(ui, "form-date", &mut form.date, lang, first, today, 240.0, p);
        if err.date {
            ui.label(RichText::new(t.err_date).font(theme::reg(12.0)).color(p.negative));
        } else {
            theme::hint(ui, t.f_date_hint, p);
        }
    });
    ui.add_space(10.0);

    // Category
    widgets::field(ui, t.f_category, p, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let label = if form.adding_category { t.f_new_category.to_string() } else { strings::category_name(lang, &form.category) };
            widgets::combo("form-category", &label, 200.0).show_ui(ui, |ui| {
                let none = form.category.is_empty() && !form.adding_category;
                if ui.selectable_label(none, RichText::new(t.no_category).font(theme::reg(13.5))).clicked() {
                    form.category.clear();
                    form.adding_category = false;
                }
                for c in cats {
                    let sel = !form.adding_category && form.category == *c;
                    if ui.selectable_label(sel, RichText::new(strings::category_name(lang, c)).font(theme::reg(13.5))).clicked() {
                        form.category = c.clone();
                        form.adding_category = false;
                    }
                }
                if ui.selectable_label(form.adding_category, RichText::new(t.f_new_category).font(theme::reg(13.5))).clicked() {
                    form.adding_category = true;
                }
            });
            if form.adding_category {
                widgets::input(ui, &mut form.new_category, t.f_category_hint, 180.0);
            }
        });
    });
    ui.add_space(10.0);

    // Note
    widgets::field(ui, t.f_note, p, |ui| {
        ui.add(
            egui::TextEdit::multiline(&mut form.note)
                .desired_rows(2)
                .desired_width(full)
                .margin(egui::Margin::symmetric(12, 8))
                .font(theme::reg(14.0)),
        );
    });
}
