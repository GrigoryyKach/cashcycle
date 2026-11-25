//! A date field: a text box (several formats accepted) and a calendar popup that honours the first day of the week.

use super::widgets::{self, Icon};
use crate::fmt;
use crate::i18n::Lang;
use crate::theme::{self, Pal};
use chrono::{Datelike, NaiveDate, Weekday};
use eframe::egui::{self, Align2, Id, Sense, Stroke, Ui, pos2, vec2};

/// Days of the month grid, starting on `first`: `None` pads the cells before the 1st and after the last day.
pub fn month_grid(year: i32, month: u32, first: Weekday) -> Vec<Option<NaiveDate>> {
    let Some(d1) = NaiveDate::from_ymd_opt(year, month, 1) else { return Vec::new() };
    let lead = (7 + d1.weekday().num_days_from_monday() as i32 - first.num_days_from_monday() as i32) % 7;
    let mut cells: Vec<Option<NaiveDate>> = vec![None; lead as usize];
    let mut d = d1;
    while d.month() == month {
        cells.push(Some(d));
        d = d.succ_opt().unwrap_or(d);
        if cells.len() > 40 {
            break;
        }
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }
    cells
}

/// Previous or next month.
pub fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let n = year * 12 + month as i32 - 1 + delta;
    (n.div_euclid(12), n.rem_euclid(12) as u32 + 1)
}

/// The weekday header in the order of the grid.
pub fn weekdays(first: Weekday) -> Vec<Weekday> {
    let mut w = first;
    (0..7)
        .map(|_| {
            let cur = w;
            w = w.succ();
            cur
        })
        .collect()
}

/// Draws the text box and the calendar button. Returns the parsed date when `text` holds a valid one;
/// `picked` is true on the frame the user chose a day in the popup (and `text` was rewritten).
pub fn date_field(ui: &mut Ui, id: &str, text: &mut String, lang: Lang, first: Weekday, today: NaiveDate, width: f32, p: &Pal) -> (Option<NaiveDate>, bool) {
    let mut picked = false;
    let parsed = fmt::parse_date(text);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let resp = widgets::input(ui, text, &fmt::date_input(lang, today), width - 40.0);
        if parsed.is_none() && !text.trim().is_empty() && !resp.has_focus() {
            widgets::flag_error(ui, &resp, p);
        }
        let btn = widgets::icon_button(ui, Icon::Calendar, 34.0, p);
        let key = Id::unique(("date-month", ui.scope_id().with(id)));
        if btn.clicked() {
            let base = parsed.unwrap_or(today);
            ui.data_mut(|d| d.insert_temp(key, (base.year(), base.month())));
        }
        egui::Popup::from_toggle_button_response(&btn).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
            ui.set_width(252.0);
            let (mut y, mut m): (i32, u32) = ui.data(|d| d.get_temp(key)).unwrap_or((today.year(), today.month()));
            ui.horizontal(|ui| {
                if widgets::icon_button(ui, Icon::Left, 30.0, p).clicked() {
                    (y, m) = shift_month(y, m, -1);
                }
                let w = ui.available_width() - 34.0;
                ui.allocate_ui_with_layout(vec2(w, 30.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                    ui.label(egui::RichText::new(fmt::month_title(lang, y, m)).font(theme::medium(14.0)).color(p.text));
                });
                if widgets::icon_button(ui, Icon::Right, 30.0, p).clicked() {
                    (y, m) = shift_month(y, m, 1);
                }
            });
            ui.data_mut(|d| d.insert_temp(key, (y, m)));
            ui.add_space(4.0);
            let cell = vec2(34.0, 30.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for wd in weekdays(first) {
                    let (r, _) = ui.allocate_exact_size(cell, Sense::hover());
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, fmt::weekday_short(lang, wd), theme::reg(11.5), p.muted);
                }
            });
            for week in month_grid(y, m, first).chunks(7) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for day in week {
                        let (r, resp) = ui.allocate_exact_size(cell, Sense::click());
                        let Some(d) = day else { continue };
                        let chosen = parsed == Some(*d);
                        let t = theme::fade(ui, resp.id.with("hover"), resp.hovered());
                        let inner = r.shrink2(vec2(2.0, 1.0));
                        if chosen {
                            ui.painter().rect_filled(inner, 6.0, p.accent);
                        } else if t > 0.0 {
                            ui.painter().rect_filled(inner, 6.0, theme::alpha(p.raised, (t * 255.0) as u8));
                        }
                        if *d == today && !chosen {
                            ui.painter().rect_stroke(inner, 6.0, Stroke::new(1.0, p.accent), egui::StrokeKind::Inside);
                        }
                        let col = if chosen { p.on_accent } else { p.text };
                        ui.painter().text(pos2(r.center().x, r.center().y), Align2::CENTER_CENTER, d.day().to_string(), theme::reg(13.0), col);
                        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            *text = fmt::date_input(lang, *d);
                            picked = true;
                            ui.close();
                        }
                    }
                });
            }
        });
    });
    (fmt::parse_date(text), picked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn grid_starts_on_the_chosen_weekday() {
        // November 2026 starts on a Sunday.
        let mon = month_grid(2026, 11, Weekday::Mon);
        assert_eq!(mon.len() % 7, 0);
        assert_eq!(mon.iter().position(|c| *c == Some(d(2026, 11, 1))), Some(6));
        let sun = month_grid(2026, 11, Weekday::Sun);
        assert_eq!(sun.iter().position(|c| *c == Some(d(2026, 11, 1))), Some(0));
        let sat = month_grid(2026, 11, Weekday::Sat);
        assert_eq!(sat.iter().position(|c| *c == Some(d(2026, 11, 1))), Some(1));
        assert_eq!(mon.iter().flatten().count(), 30);
        assert_eq!(month_grid(2028, 2, Weekday::Mon).iter().flatten().count(), 29);
    }

    #[test]
    fn months_shift_across_years() {
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 6, 0), (2026, 6));
        assert_eq!(shift_month(2026, 3, -15), (2024, 12));
    }

    #[test]
    fn weekday_header_order() {
        assert_eq!(weekdays(Weekday::Sun)[0], Weekday::Sun);
        assert_eq!(weekdays(Weekday::Sat)[6], Weekday::Fri);
        assert_eq!(weekdays(Weekday::Mon).len(), 7);
    }
}
