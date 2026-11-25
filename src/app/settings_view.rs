//! The Settings page.

use super::widgets::{self, Icon};
use super::{App, View};
use crate::config::FirstDay;
use crate::fmt;
use crate::i18n;
use crate::model::rates;
use crate::nimvoro::{self, UpdateState};
use crate::support;
use crate::theme::{self, Btn, Pal};
use crate::update_ui;
use eframe::egui::{self, Align, Layout, Margin, RichText, ScrollArea, Ui};

/// A titled group: a section title (24 above, 8 below) over a card.
fn group<R>(ui: &mut Ui, p: &Pal, title: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.add_space(24.0);
    theme::section(ui, title, p);
    ui.add_space(8.0);
    theme::card(p).inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui)
    })
    .inner
}

fn switch_row(ui: &mut Ui, p: &Pal, title: &str, hint: &str, value: &mut bool) -> bool {
    let mut changed = false;
    let w = ui.available_width();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(w - 56.0);
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new(title).font(theme::reg(14.0)).color(p.text));
            if !hint.is_empty() {
                theme::hint(ui, hint, p);
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            changed = theme::switch(ui, value, p).changed();
        });
    });
    ui.add_space(8.0);
    changed
}

/// A row "Title ........ [dropdown]".
fn combo_row<T: PartialEq + Copy>(ui: &mut Ui, p: &Pal, title: &str, id: &str, current: T, options: &[(T, String)], width: f32) -> Option<T> {
    let mut picked = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).font(theme::reg(14.0)).color(p.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            picked = widgets::pick(ui, id, current, options, width);
        });
    });
    ui.add_space(8.0);
    picked
}

impl App {
    pub(super) fn settings_view(&mut self, ui: &mut Ui, p: &Pal) {
        let ctx = ui.ctx().clone();
        let t = self.s();
        ui.horizontal(|ui| {
            if widgets::icon_button(ui, Icon::Back, 36.0, p).on_hover_text(t.back).clicked() {
                self.view = View::Main;
            }
            ui.label(RichText::new(t.settings).font(theme::medium(22.0)).color(p.text));
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.dialog.is_none() {
            self.view = View::Main;
        }
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let avail = ui.available_width();
            let col_w = (avail - 12.0).clamp(300.0, 680.0);
            ui.horizontal_top(|ui| {
                ui.add_space(((avail - col_w) / 2.0).max(0.0));
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    // ── Interface
                    group(ui, p, t.group_interface, |ui| {
                        let langs: Vec<(i18n::Lang, String)> = i18n::Lang::ALL.iter().map(|l| (*l, l.native_name().to_string())).collect();
                        if let Some(l) = combo_row(ui, p, i18n::language_label(self.lang), "language", self.lang, &langs, 150.0) {
                            self.lang = l;
                            self.cfg.language = Some(l.code().to_string());
                            self.touch_cfg(&ctx);
                        }
                        let modes: Vec<(theme::ThemeMode, String)> = theme::ThemeMode::ALL.iter().map(|m| (*m, i18n::theme_name(*m, self.lang).to_string())).collect();
                        if let Some(m) = combo_row(ui, p, i18n::theme_label(self.lang), "theme", self.cfg.theme, &modes, 150.0) {
                            self.cfg.theme = m;
                            theme::apply(&ctx, m);
                            self.touch_cfg(&ctx);
                        }
                        let t = self.s();
                        let days: Vec<(FirstDay, String)> = vec![
                            (FirstDay::Monday, t.monday.to_string()),
                            (FirstDay::Sunday, t.sunday.to_string()),
                            (FirstDay::Saturday, t.saturday.to_string()),
                        ];
                        if let Some(d) = combo_row(ui, p, t.first_day, "firstday", self.cfg.first_day, &days, 150.0) {
                            self.cfg.first_day = d;
                            self.touch_cfg(&ctx);
                        }
                    });

                    // ── Money
                    let t = self.s();
                    group(ui, p, t.group_money, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(t.default_currency).font(theme::reg(14.0)).color(p.text));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut cur = self.cfg.default_currency.clone();
                                let recent = self.cfg.recent_currencies.clone();
                                if widgets::currency_combo(ui, "default-currency", &mut cur, &recent, t.f_search_currency, 150.0, p) {
                                    self.cfg.default_currency = cur.clone();
                                    self.cfg.note_currency(&cur);
                                    self.touch_cfg(&ctx);
                                }
                            });
                        });
                        ui.add_space(2.0);
                        theme::hint(ui, t.default_currency_hint, p);
                        theme::divider(ui, p);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.label(RichText::new(t.rates_source).font(theme::reg(14.0)).color(p.text));
                                let (line, warn) = self.rates_line();
                                theme::hint(ui, &line, p);
                                if warn {
                                    ui.add_space(0.0);
                                }
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::btn(ui, t.rates_refresh_now, Btn::Secondary, p).clicked() && !self.rv.fetching {
                                    self.rates.refresh(&ctx);
                                }
                            });
                        });
                    });

                    // ── Data
                    group(ui, p, t.group_data, |ui| {
                        ui.label(RichText::new(self.dir.display().to_string()).font(theme::mono(12.0)).color(p.muted));
                        ui.add_space(8.0);
                        if widgets::btn(ui, t.open_data_folder, Btn::Secondary, p).clicked() {
                            open_folder(&self.dir);
                        }
                        ui.add_space(8.0);
                        theme::hint(ui, t.data_hint, p);
                    });

                    // ── Updates
                    let u = nimvoro::strings(self.lang);
                    group(ui, p, u.updates, |ui| {
                        let mut v = self.cfg.check_updates;
                        if switch_row(ui, p, u.check_on_start, u.check_on_start_hint, &mut v) {
                            self.cfg.check_updates = v;
                            self.touch_cfg(&ctx);
                        }
                        if !update_ui::progress(ui, &self.updater, self.lang, p) {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(self.updater.status_text(self.lang)).font(theme::reg(13.0)).color(p.text2));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if let UpdateState::Available { .. } = self.updater.state() {
                                        if widgets::btn(ui, u.update_btn, Btn::Primary, p).clicked() {
                                            let saved = self.flush();
                                            if self.updater.apply_if(saved) {
                                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                            }
                                        }
                                    }
                                    if widgets::btn(ui, u.check_now, Btn::Secondary, p).clicked() {
                                        let c = ctx.clone();
                                        self.updater.check(move || c.request_repaint());
                                    }
                                });
                            });
                        }
                    });

                    // ── Support the author
                    ui.add_space(24.0);
                    support::support_section(ui, self.lang);

                    // ── About
                    let t = self.s();
                    group(ui, p, t.about, |ui| {
                        ui.horizontal(|ui| {
                            theme::mark(ui, 32.0, widgets::mark_glyph, p);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.label(RichText::new("Cashcycle").font(theme::medium(15.0)).color(p.text));
                                theme::meta(ui, &format!("{} {}", t.version, env!("CARGO_PKG_VERSION")), p);
                            });
                        });
                        ui.add_space(8.0);
                        if widgets::btn(ui, crate::REPO_URL, Btn::Link, p).clicked() {
                            ctx.open_url(egui::OpenUrl::new_tab(crate::REPO_URL));
                        }
                        ui.add_space(4.0);
                        if widgets::btn(ui, t.rates_credit, Btn::Link, p).on_hover_text(rates::ATTRIBUTION_URL).clicked() {
                            ctx.open_url(egui::OpenUrl::new_tab(rates::ATTRIBUTION_URL));
                        }
                        if let Some(r) = &self.rv.rates {
                            theme::hint(ui, &format!("{}: {}", t.rates_updated, fmt::moment(self.lang, r.fetched)), p);
                        }
                    });
                    ui.add_space(24.0);
                });
            });
        });
    }
}

/// Opens a folder in the file manager.
fn open_folder(dir: &std::path::Path) {
    let _ = std::fs::create_dir_all(dir);
    let _ = std::process::Command::new(if cfg!(windows) { "explorer.exe" } else { "xdg-open" }).arg(dir).spawn();
}
