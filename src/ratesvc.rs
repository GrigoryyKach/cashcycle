//! Keeps the exchange rates fresh in the background: a download on start, again when the figures are older than
//! six hours (checked while the app runs) and on demand. The disk cache is only the fallback when there is no connection.

use crate::model::rates::{self, Rates};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A failed download is retried no sooner than this (seconds).
const RETRY_SECS: i64 = 15 * 60;

#[derive(Default)]
struct Inner {
    rates: Option<Rates>,
    fetching: bool,
    /// The last download failed (no connection, or the source is down).
    failed: bool,
    last_attempt: i64,
}

/// What the interface needs to draw this frame.
#[derive(Clone, Default, Debug)]
pub struct View {
    pub rates: Option<Rates>,
    pub fetching: bool,
    pub failed: bool,
}

#[derive(Clone)]
pub struct RatesSvc {
    inner: Arc<Mutex<Inner>>,
    dir: PathBuf,
    /// Demo mode: nothing is downloaded and nothing is written.
    frozen: bool,
}

impl RatesSvc {
    pub fn new(dir: PathBuf, initial: Option<Rates>, frozen: bool) -> Self {
        let inner = Inner { rates: initial, ..Default::default() };
        Self { inner: Arc::new(Mutex::new(inner)), dir, frozen }
    }

    pub fn view(&self) -> View {
        let g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        View { rates: g.rates.clone(), fetching: g.fetching, failed: g.failed }
    }

    /// Demo and screenshot helper: force the failed flag.
    pub fn set_failed(&self, failed: bool) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).failed = failed;
    }

    /// Starts a download unless one is already running.
    pub fn refresh(&self, ctx: &eframe::egui::Context) {
        if self.frozen {
            return;
        }
        {
            let mut g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if g.fetching {
                return;
            }
            g.fetching = true;
            g.last_attempt = rates::now_unix();
        }
        let me = self.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = rates::fetch();
            {
                let mut g = me.inner.lock().unwrap_or_else(|e| e.into_inner());
                g.fetching = false;
                match result {
                    Ok(r) => {
                        rates::save_cache(&me.dir, &r);
                        g.rates = Some(r);
                        g.failed = false;
                    }
                    Err(_) => g.failed = true,
                }
            }
            ctx.request_repaint();
        });
    }

    /// Call every frame: refreshes when there are no rates or they are older than six hours.
    /// After a failure it waits a quarter of an hour before trying again.
    pub fn keep_fresh(&self, ctx: &eframe::egui::Context) {
        if self.frozen {
            return;
        }
        let now = rates::now_unix();
        let due = {
            let g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            !g.fetching && is_due(g.rates.as_ref(), g.last_attempt, now)
        };
        if due {
            self.refresh(ctx);
        }
    }
}

/// Whether a refresh is due: no rates, or older than six hours, and the last try was long enough ago.
pub fn is_due(rates: Option<&Rates>, last_attempt: i64, now: i64) -> bool {
    let stale = rates.is_none_or(|r| r.is_stale(now));
    stale && (last_attempt == 0 || now - last_attempt >= RETRY_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::rates::tests::sample;

    #[test]
    fn refresh_schedule() {
        let r = sample(); // fetched at 2_000
        assert!(!is_due(Some(&r), 0, 2_000 + 3600));
        assert!(is_due(Some(&r), 0, 2_000 + rates::MAX_AGE_SECS + 1));
        // a recent failed attempt postpones the retry
        assert!(!is_due(Some(&r), 2_000 + rates::MAX_AGE_SECS, 2_000 + rates::MAX_AGE_SECS + 60));
        assert!(is_due(Some(&r), 2_000 + rates::MAX_AGE_SECS, 2_000 + rates::MAX_AGE_SECS + RETRY_SECS));
        assert!(is_due(None, 0, 5));
        assert!(!is_due(None, 100, 160));
    }

    #[test]
    fn frozen_service_never_fetches() {
        let svc = RatesSvc::new(std::env::temp_dir(), None, true);
        let ctx = eframe::egui::Context::default();
        svc.keep_fresh(&ctx);
        svc.refresh(&ctx);
        assert!(!svc.view().fetching && svc.view().rates.is_none());
    }
}
