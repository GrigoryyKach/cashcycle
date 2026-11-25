#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod demo;
#[allow(dead_code)]
mod fmt;
#[allow(dead_code)]
mod i18n;
mod model;
#[allow(dead_code)]
mod nimvoro;
mod ratesvc;
#[allow(dead_code)]
mod selfupdate;
mod shot;
mod strings;
mod support;
#[allow(dead_code)]
mod theme;
#[allow(dead_code)]
mod update_ui;

use eframe::egui;

/// The repository, kept in one place (About page and the update check).
pub const REPO_URL: &str = "https://github.com/GrigoryyKach/cashcycle";

fn main() -> eframe::Result<()> {
    selfupdate::startup("cashcycle");
    let args: Vec<String> = std::env::args().collect();
    let opts = app::Opts::parse(&args);
    let dir = config::data_dir();
    let cfg = if opts.demo { config::Config::default() } else { config::Config::load_from(&dir) };

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Cashcycle")
        .with_app_id("local.cashcycle")
        .with_min_inner_size([940.0, 640.0])
        .with_icon(std::sync::Arc::new(app_icon()));
    let mut centered = true;
    let size = args.iter().position(|a| a == "--size").and_then(|i| args.get(i + 1)).and_then(|s| {
        let (w, h) = s.split_once('x')?;
        Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?])
    });
    match (size, cfg.window) {
        (Some(s), _) => viewport = viewport.with_inner_size(s),
        (None, Some(w)) if w.w >= 940.0 && w.h >= 640.0 => {
            viewport = viewport.with_inner_size([w.w, w.h]);
            if let (Some(x), Some(y)) = (w.x, w.y) {
                if x > -20000.0 && y > -20000.0 && x < 20000.0 && y < 20000.0 {
                    viewport = viewport.with_position([x, y]);
                    centered = false;
                }
            }
        }
        _ => viewport = viewport.with_inner_size([1180.0, 780.0]),
    }
    let options = eframe::NativeOptions { viewport, centered, ..Default::default() };
    eframe::run_native("Cashcycle", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, cfg, dir, opts)))))
}

/// Window icon from `assets/icon.png` (the same artwork that is embedded in the exe).
fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).unwrap_or_default()
}
