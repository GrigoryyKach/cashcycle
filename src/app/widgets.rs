//! Drawing helpers: icons, buttons, dropdowns, tiles, modals and the footers of dialogs.

use crate::model::currency;
use crate::theme::{self, Btn, Pal};
use eframe::egui::{
    self, text::LayoutJob, Align2, Color32, CursorIcon, FontId, Frame, Id, Margin, Order, Painter, Pos2, Rect, Response, Sense,
    Stroke, StrokeKind, TextFormat, Ui, pos2, vec2,
};
use std::sync::Arc;

pub fn rgb(c: u32) -> Color32 {
    theme::hex(c)
}

/// Text colour that reads well on top of `bg`.
pub fn on_color(bg: Color32) -> Color32 {
    let l = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if l > 150.0 { theme::hex(0x141415) } else { Color32::WHITE }
}

// ───────────────────────── category colours ─────────────────────────

/// Colours of the categories in order: Video, Music, Software, Cloud, Gaming, Other, then user-defined ones.
const CATEGORY_COLORS: [u32; 10] = [0xe5484d, 0x9a7bff, 0x5b9cf5, 0x2fbfa5, 0xf2803a, 0x8a8a93, 0xec6aa8, 0xe0ac2b, 0x22b8cf, 0xb08968];

/// The colour of a category slot (see `Data::category_slot`); no category is a neutral grey.
pub fn category_color(slot: usize, p: &Pal) -> Color32 {
    if slot == usize::MAX {
        return theme::mix(p.muted, p.raised, 0.35);
    }
    rgb(CATEGORY_COLORS[slot % CATEGORY_COLORS.len()])
}

/// The first-letter tile of a subscription.
pub fn tile(painter: &Painter, rect: Rect, name: &str, color: Color32, muted: bool, p: &Pal) {
    let (fill, text) = if muted { (p.raised, p.muted) } else { (color, on_color(color)) };
    painter.rect_filled(rect, 8.0, fill);
    let letter: String = name.trim().chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "?".into());
    painter.text(rect.center(), Align2::CENTER_CENTER, letter, theme::medium(rect.height() * 0.45), text);
}

// ───────────────────────── text ─────────────────────────

/// A one-line galley that ends in "…" instead of overflowing `max_w`.
pub fn fit(painter: &Painter, text: &str, font: FontId, color: Color32, max_w: f32) -> Arc<egui::Galley> {
    let mut job = LayoutJob::single_section(text.to_string(), TextFormat { font_id: font, color, ..Default::default() });
    job.wrap.max_width = max_w.max(8.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    painter.layout_job(job)
}

// ───────────────────────── icons ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Gear,
    Link,
    Left,
    Right,
    Back,
    Calendar,
}

/// Draws a line icon centred at `c` within a `size`×`size` box.
pub fn icon(p: &Painter, ic: Icon, c: Pos2, size: f32, col: Color32) {
    let s = size / 2.0;
    let st = Stroke::new(1.5, col);
    let line = |a: Pos2, b: Pos2| p.line_segment([a, b], st);
    match ic {
        Icon::Gear => {
            p.circle_stroke(c, s * 0.45, st);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let d = vec2(a.cos(), a.sin());
                line(c + d * (s * 0.68), c + d * (s * 0.98));
            }
            p.circle_stroke(c, s * 0.75, st);
        }
        Icon::Link => {
            // The "open in browser" arrow: a box with an arrow leaving its corner.
            let b = Rect::from_center_size(c + vec2(-s * 0.1, s * 0.1), vec2(s * 1.3, s * 1.3));
            line(b.left_top(), b.left_bottom());
            line(b.left_bottom(), b.right_bottom());
            line(b.right_bottom(), pos2(b.right(), b.center().y + s * 0.1));
            line(b.left_top(), pos2(b.center().x - s * 0.1, b.top()));
            let t = c + vec2(s * 0.7, -s * 0.7);
            line(c + vec2(-s * 0.05, s * 0.05), t);
            line(t, t + vec2(-s * 0.55, 0.0));
            line(t, t + vec2(0.0, s * 0.55));
        }
        Icon::Left => {
            line(c + vec2(s * 0.25, -s * 0.6), c + vec2(-s * 0.3, 0.0));
            line(c + vec2(-s * 0.3, 0.0), c + vec2(s * 0.25, s * 0.6));
        }
        Icon::Right => {
            line(c + vec2(-s * 0.25, -s * 0.6), c + vec2(s * 0.3, 0.0));
            line(c + vec2(s * 0.3, 0.0), c + vec2(-s * 0.25, s * 0.6));
        }
        Icon::Back => {
            line(c + vec2(s * 0.7, 0.0), c + vec2(-s * 0.7, 0.0));
            line(c + vec2(-s * 0.7, 0.0), c + vec2(-s * 0.1, -s * 0.6));
            line(c + vec2(-s * 0.7, 0.0), c + vec2(-s * 0.1, s * 0.6));
        }
        Icon::Calendar => {
            let r = Rect::from_center_size(c + vec2(0.0, s * 0.1), vec2(s * 1.6, s * 1.45));
            p.rect_stroke(r, 3.0, st, StrokeKind::Inside);
            line(pos2(r.left(), r.top() + s * 0.45), pos2(r.right(), r.top() + s * 0.45));
            line(pos2(r.left() + s * 0.45, r.top() - s * 0.2), pos2(r.left() + s * 0.45, r.top() + s * 0.2));
            line(pos2(r.right() - s * 0.45, r.top() - s * 0.2), pos2(r.right() - s * 0.45, r.top() + s * 0.2));
        }
    }
}

/// A square quiet icon button (hover fill, pointing hand).
pub fn icon_button(ui: &mut Ui, ic: Icon, side: f32, p: &Pal) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(side, side), Sense::click());
    let t = theme::fade(ui, resp.id.with("hover"), resp.hovered());
    ui.painter().rect_filled(rect, 6.0, theme::alpha(p.raised, (t * 255.0) as u8));
    icon(ui.painter(), ic, rect.center(), side * 0.55, theme::mix(p.text2, p.text, t));
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// A circular arrow: about 300 degrees of arc with a clear arrowhead at its end, turned by `turn` radians.
pub fn refresh_glyph(p: &Painter, c: Pos2, size: f32, col: Color32, turn: f32) {
    let r = size * 0.36;
    let st = Stroke::new(1.6, col);
    let sweep = 300.0_f32.to_radians();
    let a0 = -std::f32::consts::FRAC_PI_2 + 0.9 + turn; // the gap sits at the top right
    let a1 = a0 + sweep;
    let n = 28;
    let at = |a: f32| c + vec2(a.cos(), a.sin()) * r;
    let pts: Vec<Pos2> = (0..=n).map(|i| at(a0 + sweep * i as f32 / n as f32)).collect();
    p.add(egui::Shape::line(pts, st));
    // The arrowhead: tip a little past the end of the arc, wings 150 degrees back from the direction of travel.
    let tip = at(a1);
    let dir = vec2(-a1.sin(), a1.cos());
    let head = size * 0.30;
    for sign in [-1.0_f32, 1.0] {
        let ang = 150.0_f32.to_radians() * sign;
        let (sn, cs) = ang.sin_cos();
        let wing = vec2(dir.x * cs - dir.y * sn, dir.x * sn + dir.y * cs) * head;
        p.line_segment([tip, tip + wing], st);
    }
}

/// The refresh button: a circular arrow that turns and is disabled while `busy`. Returns `true` when clicked.
pub fn refresh_button(ui: &mut Ui, side: f32, busy: bool, tip: &str, p: &Pal) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(side, side), if busy { Sense::hover() } else { Sense::click() });
    let t = if busy { 0.0 } else { theme::fade(ui, resp.id.with("hover"), resp.hovered()) };
    ui.painter().rect_filled(rect, 6.0, theme::alpha(p.raised, (t * 255.0) as u8));
    let (col, turn) = if busy {
        ui.ctx().request_repaint();
        (p.accent, ui.input(|i| i.time) as f32 * std::f32::consts::TAU * 0.9)
    } else {
        (theme::mix(p.text2, p.text, t), 0.0)
    };
    refresh_glyph(ui.painter(), rect.center(), side * 0.55, col, turn);
    let resp = resp.on_hover_text(tip);
    if busy { false } else { resp.on_hover_cursor(CursorIcon::PointingHand).clicked() }
}

/// The official artwork, pre-rendered from `tasks/icons/cashcycle` at the sizes that land on whole pixels at 1x,
/// 1.25x, 1.5x, 2x and 3x for a 32 px logo (the same master as the exe and window icon).
const ICONS: [(u32, &[u8]); 5] = [
    (32, include_bytes!("../../assets/icons/icon-32.png")),
    (40, include_bytes!("../../assets/icons/icon-40.png")),
    (48, include_bytes!("../../assets/icons/icon-48.png")),
    (64, include_bytes!("../../assets/icons/icon-64.png")),
    (128, include_bytes!("../../assets/icons/icon-128.png")),
];

/// Draws the app icon `size` points wide from the real artwork: the pre-rendered size nearest to this screen's
/// density, placed on the pixel grid so its edges stay sharp.
pub fn app_icon(ui: &mut Ui, size: f32) -> Response {
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let want = (size * ppp).round().max(1.0) as u32;
    let (px, bytes) = ICONS.iter().min_by_key(|(n, _)| ((*n as i64 - want as i64).abs(), *n < want)).copied().unwrap_or(ICONS[0]);
    let id = Id::unique(("app-icon", px));
    let tex: egui::TextureHandle = match ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        Some(t) => t,
        None => {
            let img = eframe::icon_data::from_png_bytes(bytes).unwrap_or_default();
            let color = egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &img.rgba);
            let t = ctx.load_texture(format!("app-icon-{px}"), color, egui::TextureOptions::LINEAR);
            ctx.data_mut(|d| d.insert_temp(id, t.clone()));
            t
        }
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let min = pos2((rect.min.x * ppp).round() / ppp, (rect.min.y * ppp).round() / ppp);
    let drawn = if px == want { px as f32 / ppp } else { size };
    ui.painter().image(tex.id(), Rect::from_min_size(min, vec2(drawn, drawn)), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    resp
}

// ───────────────────────── buttons ─────────────────────────

/// `theme::button` derives its id from the label alone, so two buttons with the same text in one frame share
/// one id. This wrapper salts the id with the button's position, so every button is independent.
pub fn btn(ui: &mut Ui, label: &str, kind: Btn, p: &Pal) -> Response {
    let pos = ui.cursor().min;
    ui.push_id(("btn-at", pos.x as i32, pos.y as i32), |ui| theme::button(ui, label, kind, p)).inner
}

// ───────────────────────── fields and dropdowns ─────────────────────────

/// Single-line text input in the suite style.
pub fn input(ui: &mut Ui, text: &mut String, hint: &str, width: f32) -> Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .desired_width(width)
            .margin(Margin::symmetric(12, 8))
            .font(theme::reg(14.0)),
    )
}

/// Marks a field as wrong: a negative outline over it.
pub fn flag_error(ui: &Ui, resp: &Response, p: &Pal) {
    ui.painter().rect_stroke(resp.rect.expand(1.0), theme::R_CONTROL as f32, Stroke::new(1.5, p.negative), StrokeKind::Outside);
}

fn chevron(ui: &Ui, rect: Rect, color: Color32) {
    theme::icon_chevron(ui.painter(), rect.center(), color, true);
}

/// A dropdown button in the suite style: text on the left, a chevron on the same row.
pub fn combo(id: impl std::hash::Hash + std::fmt::Debug, text: &str, width: f32) -> egui::ComboBox<'static> {
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(egui::RichText::new(text.to_string()).font(theme::reg(14.0)))
        .icon(|ui, rect, visuals, _open| chevron(ui, rect, visuals.fg_stroke.color))
}

/// A simple dropdown over a list of options. Returns the picked value.
pub fn pick<T: PartialEq + Copy>(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, current: T, options: &[(T, String)], width: f32) -> Option<T> {
    let mut picked = None;
    let label = options.iter().find(|o| o.0 == current).map(|o| o.1.as_str()).unwrap_or("");
    combo(id, label, width).show_ui(ui, |ui| {
        for (v, name) in options {
            if ui.selectable_label(*v == current, egui::RichText::new(name).font(theme::reg(13.5))).clicked() {
                picked = Some(*v);
            }
        }
    });
    picked
}

/// The currency dropdown: UAH, USD and EUR on top, then every other one. A search box filters by code, name or symbol.
/// Returns `true` when another currency was picked.
pub fn currency_combo(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug + Copy, cur: &mut String, recent: &[String], search_hint: &str, width: f32, p: &Pal) -> bool {
    let label = format!("{}  {}", cur, currency::symbol(cur));
    let key = Id::unique(("currency-query", ui.scope_id().with(id)));
    let mut changed = false;
    let out = combo(id, &label, width).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show_ui(ui, |ui| {
        ui.set_min_width(240.0);
        let mut q: String = ui.data_mut(|d| d.get_temp(key)).unwrap_or_default();
        let first = !ui.data(|d| d.get_temp::<bool>(key.with("open"))).unwrap_or(false);
        let r = input(ui, &mut q, search_hint, 232.0);
        if first {
            r.request_focus();
            ui.data_mut(|d| d.insert_temp(key.with("open"), true));
        }
        ui.data_mut(|d| d.insert_temp(key, q.clone()));
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            let mut shown = 0;
            let frequent = currency::frequent(recent);
            for (i, c) in currency::ordered(recent).into_iter().enumerate() {
                if !currency::matches(c, &q) {
                    continue;
                }
                if i == frequent && shown > 0 && q.trim().is_empty() {
                    theme::hairline(ui, p);
                    ui.add_space(4.0);
                }
                shown += 1;
                let text = format!("{}   {}   {}", c.code, c.symbol, c.name);
                if ui.selectable_label(c.code == cur.as_str(), egui::RichText::new(text).font(theme::reg(13.5))).clicked() {
                    *cur = c.code.to_string();
                    changed = true;
                    ui.data_mut(|d| {
                        d.remove_temp::<String>(key);
                        d.remove_temp::<bool>(key.with("open"));
                    });
                    ui.close();
                }
            }
            if shown == 0 {
                theme::hint(ui, "-", p);
            }
        });
    });
    if out.inner.is_none() {
        ui.data_mut(|d| {
            d.remove_temp::<String>(key);
            d.remove_temp::<bool>(key.with("open"));
        });
    }
    changed
}

// ───────────────────────── modals ─────────────────────────

/// Dimmed backdrop plus a centred card. Returns `true` when the backdrop was clicked.
pub fn modal(ctx: &egui::Context, p: &Pal, name: &str, generation: u64, width: f32, add: impl FnOnce(&mut Ui)) -> bool {
    // A generation number that changes on every opening gives the backdrop and the card fresh ids, so a reopened
    // modal always has the card on top of the backdrop.
    let name = format!("{name}-{generation}");
    let screen = ctx.content_rect();
    let dim = egui::Area::new(Id::unique((&name, "dim"))).order(Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
        let r = ui.allocate_rect(screen, Sense::click());
        ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(if p.dark { 150 } else { 90 }));
        r
    });
    egui::Area::new(Id::unique(&name)).order(Order::Foreground).anchor(Align2::CENTER_CENTER, vec2(0.0, -10.0)).show(ctx, |ui| {
        Frame::new()
            .fill(p.card)
            .stroke(Stroke::new(1.0, p.line_hi))
            .corner_radius(theme::R_WINDOW)
            .inner_margin(Margin::same(24))
            .shadow(theme::shadow_for(p))
            .show(ui, |ui| {
                ui.set_width(width);
                add(ui);
            });
    });
    dim.inner.clicked()
}

/// What was clicked in a modal footer.
#[derive(Default)]
pub struct FooterOut {
    pub left: bool,
    /// Index into the `secondary` labels.
    pub secondary: Option<usize>,
    pub primary: bool,
}

/// The one footer every modal uses: all buttons are 36 px tall with the same minimum width, the accent button is
/// rightmost, secondary buttons sit to its left, an optional button sits at the far left.
pub fn footer(ui: &mut Ui, p: &Pal, left: Option<&str>, secondary: &[&str], primary: &str, enabled: bool) -> FooterOut {
    const H: f32 = 36.0;
    const GAP: f32 = 8.0;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), H), Sense::hover());
    let painter = ui.painter().clone();
    let width = |label: &str| (painter.layout_no_wrap(label.to_string(), theme::medium(14.0), p.text).size().x + 40.0).max(112.0);
    let mut out = FooterOut::default();
    let mut x = row.right();
    let w = width(primary);
    x -= w;
    let rect = Rect::from_min_size(pos2(x, row.top()), vec2(w, H));
    let id = ui.scope_id().with(("footer-primary", primary));
    if enabled {
        if theme::button_at(ui, rect, id, primary, Btn::Primary, p).clicked() {
            out.primary = true;
        }
    } else {
        ui.interact(rect, id, Sense::hover());
        painter.rect(rect, theme::R_CONTROL as f32, p.raised, Stroke::new(1.0, p.line), StrokeKind::Inside);
        painter.text(rect.center(), Align2::CENTER_CENTER, primary, theme::medium(14.0), p.muted);
    }
    for (i, label) in secondary.iter().enumerate().rev() {
        let w = width(label);
        x -= GAP + w;
        let rect = Rect::from_min_size(pos2(x, row.top()), vec2(w, H));
        if theme::button_at(ui, rect, ui.scope_id().with(("footer-secondary", i, *label)), label, Btn::Secondary, p).clicked() {
            out.secondary = Some(i);
        }
    }
    if let Some(label) = left {
        let rect = Rect::from_min_size(row.min, vec2(width(label), H));
        if theme::button_at(ui, rect, ui.scope_id().with(("footer-left", label)), label, Btn::Secondary, p).clicked() {
            out.left = true;
        }
    }
    out
}

/// What was clicked in a destructive confirmation.
#[derive(Default)]
pub struct ConfirmOut {
    pub cancel: bool,
    pub confirm: bool,
}

/// Footer of a destructive confirmation. Cancel is the default (accent outline, rightmost, Enter and Escape);
/// the destructive action sits to its left in the negative colour, so a habitual click on the right is safe.
pub fn destructive_footer(ui: &mut Ui, p: &Pal, cancel: &str, confirm: &str) -> ConfirmOut {
    const H: f32 = 36.0;
    const GAP: f32 = 8.0;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), H), Sense::hover());
    let painter = ui.painter().clone();
    let width = |label: &str| (painter.layout_no_wrap(label.to_string(), theme::medium(14.0), p.text).size().x + 40.0).max(112.0);
    let mut out = ConfirmOut::default();
    let cw = width(cancel);
    let cancel_rect = Rect::from_min_size(pos2(row.right() - cw, row.top()), vec2(cw, H));
    if theme::button_at(ui, cancel_rect, ui.scope_id().with(("destructive-cancel", cancel)), cancel, Btn::Secondary, p).clicked() {
        out.cancel = true;
    }
    painter.rect_stroke(cancel_rect, theme::R_CONTROL as f32, Stroke::new(1.5, p.accent), StrokeKind::Inside);
    let dw = width(confirm);
    let rect = Rect::from_min_size(pos2(cancel_rect.left() - GAP - dw, row.top()), vec2(dw, H));
    let id = ui.scope_id().with(("destructive-confirm", confirm));
    let resp = ui.interact(rect, id, Sense::click());
    let t = theme::fade(ui, id.with("hover"), resp.hovered());
    let fill = theme::mix(p.negative, if p.dark { Color32::WHITE } else { Color32::BLACK }, t * 0.14);
    let text = if p.dark { theme::hex(0x141415) } else { Color32::WHITE };
    painter.rect_filled(rect, theme::R_CONTROL as f32, fill);
    painter.text(rect.center(), Align2::CENTER_CENTER, confirm, theme::medium(14.0), text);
    if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        out.confirm = true;
    }
    out
}

/// A labelled field block: a small caption over whatever `add` draws.
pub fn field<R>(ui: &mut Ui, caption: &str, p: &Pal, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.label(egui::RichText::new(caption).font(theme::reg(12.5)).color(p.text2));
    ui.add_space(2.0);
    add(ui)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Modifiers, PointerButton, RawInput};

    fn frame(ctx: &egui::Context, events: Vec<Event>, t: f64) -> Option<bool> {
        let mut input = RawInput::default();
        input.screen_rect = Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)));
        input.time = Some(t);
        input.events = events;
        let mut clicked = None;
        let mut out = ctx.run_pass(input, |ctx| {
            clicked = Some(modal(ctx, &Pal::dark(), "test-modal", 1, 300.0, |ui| {
                ui.label("content");
            }));
        });
        out.textures_delta.clear();
        clicked
    }

    fn click(ctx: &egui::Context, at: Pos2, t: &mut f64) -> bool {
        let mut hit = false;
        for events in [
            vec![Event::PointerMoved(at)],
            vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }],
            vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
            vec![],
        ] {
            *t += 0.05;
            hit |= frame(ctx, events, *t).unwrap_or(false);
        }
        hit
    }

    /// The bug: clicking the dimmed backdrop raised the backdrop above the card (same layer order), so the card
    /// stopped receiving input and the screen stayed dimmed. The card must stay on top however often the
    /// backdrop is clicked, and a click on the backdrop must be reported to the caller each time.
    #[test]
    fn clicking_the_backdrop_never_buries_the_card() {
        let ctx = egui::Context::default();
        let mut t = 0.0;
        for _ in 0..3 {
            t += 0.05;
            frame(&ctx, vec![], t);
        }
        let card = pos2(400.0, 290.0);
        let top = |ctx: &egui::Context, at: Pos2| ctx.layer_id_at(at).map(|l| l.order);
        assert_eq!(top(&ctx, card), Some(Order::Foreground));
        assert_eq!(top(&ctx, pos2(20.0, 20.0)), Some(Order::Middle), "the backdrop covers the page");
        for _ in 0..4 {
            assert!(click(&ctx, pos2(20.0, 20.0), &mut t), "the click outside is reported");
            assert_eq!(top(&ctx, card), Some(Order::Foreground), "the card is still on top after the click");
        }
        // A click on the card itself is not a backdrop click.
        assert!(!click(&ctx, card, &mut t));
    }
}
