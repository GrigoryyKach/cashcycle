//! The main page: totals, the subscription list, upcoming payments and the category breakdown.

use super::widgets::{self, Icon};
use super::{card_margin, current_totals, App, Dialog, Form, View};
use crate::fmt;
use crate::model::calc::{self, Sort, Totals};
use crate::model::types::Subscription;
use crate::strings;
use crate::theme::{self, Btn, Pal, Seg};
use eframe::egui::{self, Align, Align2, Layout, Rect, RichText, ScrollArea, Sense, Stroke, Ui, pos2, vec2};

const ROW_HEIGHT: f32 = 62.0;
const SIDE_W: f32 = 340.0;

#[derive(Default)]
struct RowOut {
    edit: bool,
    link: bool,
    pause: bool,
    resume: bool,
}

impl App {
    pub(super) fn main_view(&mut self, ui: &mut Ui, p: &Pal) {
        self.top_bar(ui, p);
        ui.add_space(14.0);
        if self.data.subs.is_empty() {
            self.empty_state(ui, p);
            return;
        }
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let w = ui.available_width() - 4.0;
            ui.set_width(w);
            let totals = current_totals(self);
            self.cards_row(ui, p, w, &totals);
            ui.add_space(16.0);
            let gap = 16.0;
            let left_w = w - SIDE_W - gap;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                ui.vertical(|ui| {
                    ui.set_width(left_w);
                    self.list_column(ui, p, left_w);
                });
                ui.vertical(|ui| {
                    ui.set_width(SIDE_W);
                    self.side_column(ui, p, SIDE_W);
                });
            });
            ui.add_space(28.0);
        });
    }

    fn top_bar(&mut self, ui: &mut Ui, p: &Pal) {
        let t = self.s();
        ui.horizontal(|ui| {
            widgets::app_icon(ui, 32.0);
            ui.label(RichText::new("Cashcycle").font(theme::medium(20.0)).color(p.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::icon_button(ui, Icon::Gear, ROW_HEIGHT_ICON, p).on_hover_text(t.settings).clicked() {
                    self.view = View::Settings;
                }
                if !self.data.subs.is_empty() && widgets::btn(ui, t.add_sub, Btn::Primary, p).clicked() {
                    self.open(Dialog::Edit(Form::new(&self.cfg.default_currency, self.lang, self.today)));
                }
            });
        });
    }

    fn empty_state(&mut self, ui: &mut Ui, p: &Pal) {
        let t = self.s();
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            theme::card(p).inner_margin(egui::Margin::symmetric(40, 32)).show(ui, |ui| {
                ui.set_width(440.0);
                ui.vertical_centered(|ui| {
                    widgets::app_icon(ui, 64.0);
                    ui.add_space(10.0);
                    ui.label(RichText::new(t.empty_title).font(theme::medium(22.0)).color(p.text));
                    ui.add_space(6.0);
                    ui.add(egui::Label::new(RichText::new(t.empty_text).font(theme::reg(14.0)).color(p.text2)).wrap());
                    ui.add_space(18.0);
                    if widgets::btn(ui, t.empty_button, Btn::Primary, p).clicked() {
                        self.open(Dialog::Edit(Form::new(&self.cfg.default_currency, self.lang, self.today)));
                    }
                });
            });
        });
    }

    // ───────────────────────── cards ─────────────────────────

    fn cards_row(&mut self, ui: &mut Ui, p: &Pal, w: f32, totals: &Totals) {
        let t = self.s();
        let gap = 12.0;
        let rest = w - 2.0 * gap;
        let (w1, w2) = (rest * 0.44, rest * 0.28);
        let w3 = rest - w1 - w2;
        let h = 162.0;
        let cur = self.total_currency().to_string();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            card_box(ui, p, w1, h, |ui| self.total_card(ui, p, totals, &cur));
            card_box(ui, p, w2, h, |ui| {
                theme::section(ui, t.yearly_total, p);
                ui.add_space(4.0);
                let text = if totals.converted { self.money(totals.yearly, &cur) } else { "-".to_string() };
                ui.label(RichText::new(text).font(theme::medium(28.0)).color(p.text));
                ui.add_space(4.0);
                theme::hint(ui, t.per_year_hint, p);
            });
            card_box(ui, p, w3, h, |ui| {
                theme::section(ui, t.status_card, p);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 28.0;
                    for (n, label) in [(totals.active, t.active), (totals.paused, t.paused)] {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(RichText::new(n.to_string()).font(theme::medium(28.0)).color(if n == 0 && label == t.paused { p.muted } else { p.text }));
                            theme::hint(ui, label, p);
                        });
                    }
                });
            });
        });
    }

    fn total_card(&mut self, ui: &mut Ui, p: &Pal, totals: &Totals, cur: &str) {
        let t = self.s();
        theme::section(ui, t.monthly_total, p);
        ui.add_space(2.0);
        if totals.converted {
            ui.label(RichText::new(self.money(totals.monthly, cur)).font(theme::medium(34.0)).color(p.text));
        } else {
            ui.label(RichText::new(t.total_unavailable).font(theme::medium(22.0)).color(p.text2));
            let parts: Vec<String> = totals.per_currency.iter().take(3).map(|(c, m, _)| self.money(*m, c)).collect();
            theme::hint(ui, &format!("{}: {}", t.per_currency, parts.join("  ·  ")), p);
            if self.rv.rates.is_none() {
                theme::hint(ui, t.rates_none_hint, p);
            }
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let mut picked = cur.to_string();
            let recent = self.cfg.recent_currencies.clone();
            if widgets::currency_combo(ui, "total-currency", &mut picked, &recent, t.f_search_currency, 100.0, p) {
                self.cfg.total_currency = picked.clone();
                self.cfg.note_currency(&picked);
                let ctx = ui.ctx().clone();
                self.touch_cfg(&ctx);
            }
            let busy = self.rv.fetching;
            if widgets::refresh_button(ui, 30.0, busy, t.rates_refresh, p) {
                let ctx = ui.ctx().clone();
                self.rates.refresh(&ctx);
            }
            let (line, warn) = self.rates_line();
            let col = if warn { p.warning } else { p.muted };
            ui.add(egui::Label::new(RichText::new(&line).font(theme::reg(12.0)).color(col)).truncate()).on_hover_text(&line);
        });
        if !totals.missing.is_empty() {
            let text = t.rates_missing.replace("{codes}", &totals.missing.join(", "));
            ui.add(egui::Label::new(RichText::new(text).font(theme::reg(12.0)).color(p.warning)).wrap());
        }
    }

    /// The status of the exchange rates in words, and whether it is a warning.
    pub(super) fn rates_line(&self) -> (String, bool) {
        let t = self.s();
        let rv = &self.rv;
        if rv.fetching {
            return (t.rates_refreshing.to_string(), false);
        }
        match &rv.rates {
            None => (t.rates_none.to_string(), true),
            Some(r) if rv.failed => (t.rates_offline.replace("{when}", &date_of(self.lang, r.fetched)), true),
            Some(r) => (t.rates_from.replace("{when}", &fmt::moment(self.lang, r.fetched)), false),
        }
    }

    // ───────────────────────── list ─────────────────────────

    fn list_column(&mut self, ui: &mut Ui, p: &Pal, w: f32) {
        let t = self.s();
        let cur = self.total_currency().to_string();
        // Search and sort.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let items = [Seg { label: t.sort_next, icon: None }, Seg { label: t.sort_price, icon: None }, Seg { label: t.sort_name, icon: None }];
            let seg_w = theme::segmented_width(ui, &items, p);
            let input_w = (w - seg_w - 10.0).max(120.0);
            let resp = widgets::input(ui, &mut self.search, t.search_hint, input_w);
            let _ = resp;
            let sel = match self.cfg.sort {
                Sort::Next => 0,
                Sort::Price => 1,
                Sort::Name => 2,
            };
            if let Some(i) = ui.push_id("sort", |ui| theme::segmented(ui, &items, sel, p)).inner {
                self.cfg.sort = [Sort::Next, Sort::Price, Sort::Name][i];
                let ctx = ui.ctx().clone();
                self.touch_cfg(&ctx);
            }
        });
        ui.add_space(10.0);

        let active = calc::list(&self.data, false, &self.search, self.cfg.sort, self.today, &cur, self.rates_ref());
        let paused = calc::list(&self.data, true, &self.search, self.cfg.sort, self.today, &cur, self.rates_ref());
        if active.is_empty() && paused.is_empty() {
            theme::hint(ui, t.no_matches, p);
            return;
        }
        let mut out: Option<(String, RowOut)> = None;
        if !active.is_empty() {
            self.rows_card(ui, p, w, &active, &mut out);
        }
        if !paused.is_empty() {
            ui.add_space(18.0);
            ui.label(RichText::new(format!("{} ({})", t.paused_section, paused.len())).font(theme::medium(13.0)).color(p.text2));
            ui.add_space(8.0);
            self.rows_card(ui, p, w, &paused, &mut out);
        }
        if let Some((id, o)) = out {
            self.apply_row(ui.ctx(), &id, o);
        }
    }

    fn rows_card(&self, ui: &mut Ui, p: &Pal, w: f32, idx: &[usize], out: &mut Option<(String, RowOut)>) {
        theme::card(p).inner_margin(egui::Margin::symmetric(0, 4)).show(ui, |ui| {
            ui.set_width(w - 2.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            for (n, i) in idx.iter().enumerate() {
                let s = self.data.subs[*i].clone();
                let o = self.draw_row(ui, p, &s, w - 2.0, n == 0);
                if o.edit || o.link || o.pause || o.resume {
                    *out = Some((s.id.clone(), o));
                }
            }
        });
    }

    fn apply_row(&mut self, ctx: &egui::Context, id: &str, o: RowOut) {
        let Some(s) = self.data.sub(id).cloned() else { return };
        if o.link {
            ctx.open_url(egui::OpenUrl::new_tab(s.url.clone()));
        } else if o.pause {
            self.open(Dialog::Pause { id: id.to_string(), until_on: false, until: String::new() });
        } else if o.resume {
            let next = s.next_payment(self.today);
            self.open(Dialog::Resume { id: id.to_string(), date: fmt::date_input(self.lang, next) });
        } else if o.edit {
            self.open(Dialog::Edit(Form::from_sub(&s, self.lang)));
        }
    }

    fn draw_row(&self, ui: &mut Ui, p: &Pal, s: &Subscription, w: f32, first: bool) -> RowOut {
        let t = self.s();
        let lang = self.lang;
        let cur = self.total_currency();
        let paused = s.is_paused();
        let mut out = RowOut::default();
        let (r, resp) = ui.allocate_exact_size(vec2(w, ROW_HEIGHT), Sense::click());
        let hovered = ui.rect_contains_pointer(r) && ui.is_enabled();
        let f = theme::fade(ui, resp.id.with("hover"), hovered);
        let painter = ui.painter().clone();
        painter.rect_filled(r.shrink2(vec2(4.0, 1.0)), 8.0, theme::alpha(p.raised, (f * 255.0) as u8));
        if !first {
            painter.hline(r.x_range().shrink(16.0), r.top(), Stroke::new(1.0, p.line));
        }
        let cy = r.center().y;
        let right = r.right() - 14.0;
        let action = Rect::from_min_max(pos2(right - 96.0, cy - 14.0), pos2(right, cy + 14.0));
        let date_x = action.left() - 8.0 - 84.0;
        let price_x = date_x - 8.0 - 132.0;
        let name_x = r.left() + 14.0 + 34.0 + 12.0;
        let name_w = (price_x - 12.0 - name_x).max(40.0);

        let slot = self.data.category_slot(&s.category);
        widgets::tile(&painter, Rect::from_min_size(pos2(r.left() + 14.0, cy - 17.0), vec2(34.0, 34.0)), &s.name, widgets::category_color(slot, p), paused, p);

        let name_col = if paused { p.text2 } else { p.text };
        let sub_col = p.muted;
        let link_room = if s.url.is_empty() { 0.0 } else { 24.0 };
        let g = widgets::fit(&painter, &s.name, theme::medium(14.0), name_col, name_w - link_room);
        let gw = g.size().x;
        painter.galley(pos2(name_x, cy - 9.0 - g.size().y / 2.0), g, name_col);
        if !s.url.is_empty() {
            let ir = Rect::from_center_size(pos2(name_x + gw + 6.0 + 11.0, cy - 9.0), vec2(22.0, 22.0));
            let lr = ui.interact(ir, resp.id.with("link"), Sense::click()).on_hover_text(t.open_link).on_hover_cursor(egui::CursorIcon::PointingHand);
            let col = if lr.hovered() { p.accent } else { p.muted };
            widgets::icon(&painter, Icon::Link, ir.center(), 14.0, col);
            if lr.clicked() {
                out.link = true;
            }
        }
        let cat = strings::category_name(lang, &s.category);
        let sub_text = if s.category.is_empty() { strings::period_text(lang, s.period) } else { format!("{cat}  ·  {}", strings::period_text(lang, s.period)) };
        let g = widgets::fit(&painter, &sub_text, theme::reg(12.0), sub_col, name_w);
        painter.galley(pos2(name_x, cy + 9.0 - g.size().y / 2.0), g, sub_col);

        // Price, and the monthly equivalent in the card currency.
        let price_text = format!("{} {}", self.money(s.price, &s.currency), strings::period_short(lang, s.period));
        let g = widgets::fit(&painter, &price_text, theme::medium(13.5), if paused { p.muted } else { p.text }, 132.0);
        painter.galley(pos2(price_x, cy - 9.0 - g.size().y / 2.0), g, p.text);
        let same_monthly = s.currency.eq_ignore_ascii_case(cur) && s.period == crate::model::period::Period::Monthly;
        if !same_monthly {
            if let Some(m) = calc::monthly_in(s, cur, self.rates_ref()) {
                let text = format!("≈ {} {}", self.money(m, cur), t.short_month);
                let g = widgets::fit(&painter, &text, theme::reg(12.0), p.muted, 132.0);
                painter.galley(pos2(price_x, cy + 9.0 - g.size().y / 2.0), g, p.muted);
            }
        }

        // Next payment, or the pause state.
        let (l1, l2) = if let Some(pause) = s.paused {
            (t.paused.to_string(), pause.until.map(|u| t.paused_until.replace("{date}", &fmt::date_short(lang, u))).unwrap_or_default())
        } else {
            let next = s.next_payment(self.today);
            (fmt::date_short(lang, next), strings::relative_days(lang, (next - self.today).num_days()))
        };
        painter.text(pos2(date_x, cy - 9.0), Align2::LEFT_CENTER, l1, theme::medium(13.0), if paused { p.muted } else { p.text });
        let g = widgets::fit(&painter, &l2, theme::reg(12.0), p.muted, 84.0);
        painter.galley(pos2(date_x, cy + 9.0 - g.size().y / 2.0), g, p.muted);

        // Quick action: Pause on hover, Resume always on a paused row.
        if paused || hovered {
            let label = if paused { t.resume } else { t.pause };
            let bw = painter.layout_no_wrap(label.to_string(), theme::medium(12.0), p.text).size().x + 18.0;
            let br = Rect::from_min_max(pos2(action.right() - bw, action.top()), action.right_bottom());
            let id = resp.id.with("action");
            if theme::button_at(ui, br, id, label, Btn::Small, p).clicked() {
                if paused {
                    out.resume = true;
                } else {
                    out.pause = true;
                }
            }
        }
        if resp.clicked() && !out.link && !out.pause && !out.resume {
            out.edit = true;
        }
        out
    }

    // ───────────────────────── side column ─────────────────────────

    fn side_column(&mut self, ui: &mut Ui, p: &Pal, w: f32) {
        self.upcoming_card(ui, p, w);
        ui.add_space(16.0);
        self.category_card(ui, p, w);
    }

    fn upcoming_card(&self, ui: &mut Ui, p: &Pal, w: f32) {
        let t = self.s();
        let lang = self.lang;
        let cur = self.total_currency().to_string();
        theme::card(p).inner_margin(egui::Margin::symmetric(0, 14)).show(ui, |ui| {
            ui.set_width(w - 2.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(RichText::new(t.upcoming_title).font(theme::medium(14.0)).color(p.text));
                ui.label(RichText::new(t.upcoming_hint).font(theme::reg(12.0)).color(p.muted));
            });
            ui.add_space(6.0);
            let up = calc::upcoming(&self.data, self.today);
            if up.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    theme::hint(ui, t.nothing_upcoming, p);
                });
                return;
            }
            ui.spacing_mut().item_spacing.y = 0.0;
            for u in up.iter().take(7) {
                let Some(s) = self.data.sub(&u.id) else { continue };
                let (r, _) = ui.allocate_exact_size(vec2(w - 2.0, 44.0), Sense::hover());
                let cy = r.center().y;
                let painter = ui.painter().clone();
                painter.text(pos2(r.left() + 16.0, cy - 8.0), Align2::LEFT_CENTER, fmt::date_short(lang, u.date), theme::mono(12.0), p.text2);
                let rel = strings::relative_days(lang, (u.date - self.today).num_days());
                painter.text(pos2(r.left() + 16.0, cy + 9.0), Align2::LEFT_CENTER, rel, theme::reg(11.5), p.muted);
                let own = self.money(s.price, &s.currency);
                let conv = if s.currency.eq_ignore_ascii_case(&cur) {
                    None
                } else {
                    calc::price_in(s, &cur, self.rates_ref()).map(|d| format!("≈ {}", self.money(d, &cur)))
                };
                let amount_w = 84.0;
                let g = widgets::fit(&painter, &own, theme::medium(13.0), p.text, amount_w);
                painter.galley(pos2(r.right() - 16.0 - g.size().x, cy - 8.0 - g.size().y / 2.0), g, p.text);
                if let Some(c) = conv {
                    let g = widgets::fit(&painter, &c, theme::reg(11.5), p.muted, amount_w);
                    painter.galley(pos2(r.right() - 16.0 - g.size().x, cy + 9.0 - g.size().y / 2.0), g, p.muted);
                }
                let name_x = r.left() + 108.0;
                let g = widgets::fit(&painter, &s.name, theme::reg(13.5), p.text, (r.right() - 16.0 - amount_w - 8.0 - name_x).max(30.0));
                painter.galley(pos2(name_x, cy - g.size().y / 2.0), g, p.text);
            }
            if up.len() > 7 {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    theme::hint(ui, &t.more_upcoming.replace("{n}", &(up.len() - 7).to_string()), p);
                });
            }
        });
    }

    fn category_card(&self, ui: &mut Ui, p: &Pal, w: f32) {
        let t = self.s();
        let cur = self.total_currency().to_string();
        theme::card(p).inner_margin(card_margin()).show(ui, |ui| {
            ui.set_width(w - 34.0);
            ui.label(RichText::new(t.by_category).font(theme::medium(14.0)).color(p.text));
            ui.add_space(8.0);
            let cats = calc::by_category(&self.data, &cur, self.rates_ref());
            let sum: rust_decimal::Decimal = cats.iter().map(|c| c.1).sum();
            if cats.is_empty() || sum.is_zero() {
                theme::hint(ui, t.rates_needed, p);
                return;
            }
            let sumf = crate::model::money::to_f64(sum).max(1e-9);
            let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 12.0), Sense::hover());
            let mut x = bar.left();
            let total_gap = 2.0 * (cats.len() as f32 - 1.0);
            for (name, v) in &cats {
                let frac = (crate::model::money::to_f64(*v) / sumf) as f32;
                let seg_w = ((bar.width() - total_gap) * frac).max(2.0);
                let col = widgets::category_color(self.data.category_slot(name), p);
                ui.painter().rect_filled(Rect::from_min_size(pos2(x, bar.top()), vec2(seg_w, bar.height())), 3.0, col);
                x += seg_w + 2.0;
            }
            ui.add_space(10.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            for (name, v) in cats.iter().take(8) {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
                let cy = r.center().y;
                let col = widgets::category_color(self.data.category_slot(name), p);
                ui.painter().circle_filled(pos2(r.left() + 5.0, cy), 4.5, col);
                let pct = (crate::model::money::to_f64(*v) / sumf * 100.0).round() as i32;
                let right = format!("{}  {}%", self.money(*v, &cur), pct);
                let g = widgets::fit(ui.painter(), &right, theme::reg(12.5), p.text2, r.width() * 0.6);
                let gw = g.size().x;
                ui.painter().galley(pos2(r.right() - gw, cy - g.size().y / 2.0), g, p.text2);
                let label = strings::category_name(self.lang, name);
                let g = widgets::fit(ui.painter(), &label, theme::reg(13.0), p.text, (r.width() - gw - 28.0).max(30.0));
                ui.painter().galley(pos2(r.left() + 18.0, cy - g.size().y / 2.0), g, p.text);
            }
        });
    }
}

const ROW_HEIGHT_ICON: f32 = 36.0;

/// A card of an exact width and a minimum height; `add` draws its content.
fn card_box(ui: &mut Ui, p: &Pal, w: f32, h: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui(vec2(w, h), |ui| {
        theme::card(p).inner_margin(card_margin()).show(ui, |ui| {
            ui.set_width(w - 34.0);
            ui.set_min_height(h - 30.0);
            ui.vertical(add);
        });
    });
}

fn date_of(lang: crate::i18n::Lang, unix: i64) -> String {
    use chrono::TimeZone;
    match chrono::Local.timestamp_opt(unix, 0).single() {
        Some(t) => fmt::date_short(lang, t.date_naive()),
        None => String::new(),
    }
}

