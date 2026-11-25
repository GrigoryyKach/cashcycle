//! Reading and writing `data.json`: atomic writes, schema migration with a backup first,
//! corrupted-file handling and one backup per day.

use super::types::{Data, SCHEMA_VERSION};
use chrono::NaiveDate;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DATA_FILE: &str = "data.json";
pub const BACKUP_DIR: &str = "backups";
/// Automatic daily backups kept.
pub const KEEP_BACKUPS: usize = 14;

/// What happened while loading.
#[derive(Debug, PartialEq, Eq)]
pub enum LoadOutcome {
    /// No data file yet.
    Fresh,
    Loaded,
    /// The file was unreadable or from a newer version: it was kept aside and the app starts empty.
    Recovered { moved_to: PathBuf },
}

/// Writes through a temp file in the same folder, then renames over the target.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// Parses data text (BOM tolerant) and migrates older schemas. `Err` for broken or too-new data.
pub fn parse(text: &str) -> Result<Data, String> {
    let value: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    migrate(value)
}

/// Upgrades the JSON value to the current schema, then deserialises it.
pub fn migrate(mut value: serde_json::Value) -> Result<Data, String> {
    let obj = value.as_object_mut().ok_or("data is not an object")?;
    let version = obj.get("version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if version > SCHEMA_VERSION {
        return Err(format!("data schema {version} is newer than supported {SCHEMA_VERSION}"));
    }
    // v0 -> v1: prices were plain JSON numbers, they are exact decimal strings now; `category` could be null.
    if version < 1 {
        if let Some(subs) = obj.get_mut("subs").and_then(|s| s.as_array_mut()) {
            for s in subs.iter_mut().filter_map(|s| s.as_object_mut()) {
                if let Some(n) = s.get("price").and_then(|p| p.as_number()).map(|n| n.to_string()) {
                    s.insert("price".into(), serde_json::Value::String(n));
                }
                for key in ["category", "note", "url"] {
                    if s.get(key).is_some_and(|v| v.is_null()) {
                        s.insert(key.into(), serde_json::Value::String(String::new()));
                    }
                }
            }
        }
    }
    obj.insert("version".into(), SCHEMA_VERSION.into());
    let mut data: Data = serde_json::from_value(value).map_err(|e| e.to_string())?;
    data.normalize();
    Ok(data)
}

/// The schema version a data text was written with (0 when it has none or cannot be read).
pub fn schema_of(text: &str) -> u32 {
    serde_json::from_str::<serde_json::Value>(text.trim_start_matches('\u{feff}'))
        .ok()
        .and_then(|v| v.get("version").and_then(|v| v.as_u64()))
        .unwrap_or(0) as u32
}

/// Name of the copy kept of a file written by an older schema, made before the first save in the new one.
pub fn pre_migration_name(old_version: u32) -> String {
    format!("before-schema-{SCHEMA_VERSION}-from-v{old_version}.json")
}

/// Loads `data.json` from `dir`. A broken file is renamed to `data.corrupt-<stamp>.json`, never overwritten.
pub fn load(dir: &Path, stamp: &str) -> (Data, LoadOutcome) {
    let path = dir.join(DATA_FILE);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return (Data::default(), LoadOutcome::Fresh),
        Err(_) => return (Data::default(), quarantine(&path, dir, stamp)),
    };
    match std::str::from_utf8(&bytes).map_err(|e| e.to_string()).and_then(parse) {
        Ok(d) => {
            // An older file is upgraded in memory only. Keep its exact bytes before anything is written in the
            // new schema, so going back to an older version loses nothing.
            if let Ok(text) = std::str::from_utf8(&bytes) {
                let old = schema_of(text);
                let copy = dir.join(BACKUP_DIR).join(pre_migration_name(old));
                if old < SCHEMA_VERSION && !copy.exists() {
                    let _ = write_atomic(&copy, &bytes);
                }
            }
            (d, LoadOutcome::Loaded)
        }
        Err(_) => (Data::default(), quarantine(&path, dir, stamp)),
    }
}

fn quarantine(path: &Path, dir: &Path, stamp: &str) -> LoadOutcome {
    let target = dir.join(format!("data.corrupt-{stamp}.json"));
    match fs::rename(path, &target) {
        Ok(()) => LoadOutcome::Recovered { moved_to: target },
        // If even renaming fails, keep the file where it is and still report it.
        Err(_) => LoadOutcome::Recovered { moved_to: path.to_path_buf() },
    }
}

/// Saves the data atomically. The first save of each day also writes `backups/data-YYYY-MM-DD.json`.
pub fn save(dir: &Path, data: &Data, today: NaiveDate) -> io::Result<()> {
    let json = serde_json::to_vec_pretty(data).map_err(io::Error::other)?;
    write_atomic(&dir.join(DATA_FILE), &json)?;
    let backups = dir.join(BACKUP_DIR);
    let todays = backups.join(backup_name(today));
    if !todays.exists() {
        write_atomic(&todays, &json)?;
        rotate(&backups, KEEP_BACKUPS);
    }
    Ok(())
}

pub fn backup_name(day: NaiveDate) -> String {
    format!("data-{}.json", day.format("%Y-%m-%d"))
}

/// Keeps the newest `keep` daily backups (the pre-migration copies are never removed).
fn rotate(dir: &Path, keep: usize) {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("data-") && n.ends_with(".json"))).collect())
        .unwrap_or_default();
    files.sort();
    while files.len() > keep {
        let _ = fs::remove_file(files.remove(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::period::Period;
    use crate::model::types::tests::sample;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("cashcycle-st-{name}-{}", crate::model::types::new_id()));
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
    }

    #[test]
    fn missing_file_is_fresh() {
        let dir = tmp("fresh");
        let (d, o) = load(&dir, "x");
        assert_eq!(o, LoadOutcome::Fresh);
        assert!(d.subs.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_and_load_round_trip_with_a_daily_backup() {
        let dir = tmp("rt");
        let mut data = Data::default();
        data.subs.push(sample("Netflix", "9.99", "USD", Period::Monthly));
        save(&dir, &data, day(25)).unwrap();
        assert!(dir.join(BACKUP_DIR).join(backup_name(day(25))).exists());
        assert!(!dir.join("data.tmp").exists());
        let (back, o) = load(&dir, "x");
        assert_eq!(o, LoadOutcome::Loaded);
        assert_eq!(back, data);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn old_backups_are_rotated() {
        let dir = tmp("rot");
        let data = Data::default();
        for d in 1..=20 {
            save(&dir, &data, day(d)).unwrap();
        }
        let n = fs::read_dir(dir.join(BACKUP_DIR)).unwrap().count();
        assert_eq!(n, KEEP_BACKUPS);
        assert!(!dir.join(BACKUP_DIR).join(backup_name(day(1))).exists());
        assert!(dir.join(BACKUP_DIR).join(backup_name(day(20))).exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn version_zero_numbers_are_migrated_and_backed_up_first() {
        let dir = tmp("mig");
        let old = r#"{"subs":[{"id":"a1","name":"Spotify","price":4.99,"currency":"EUR","period":"monthly","start":"2026-11-03","category":null}]}"#;
        fs::write(dir.join(DATA_FILE), old).unwrap();
        let (d, o) = load(&dir, "x");
        assert_eq!(o, LoadOutcome::Loaded);
        assert_eq!(d.version, SCHEMA_VERSION);
        assert_eq!(d.subs[0].price.to_string(), "4.99");
        assert_eq!(d.subs[0].category, "");
        let copy = dir.join(BACKUP_DIR).join(pre_migration_name(0));
        assert_eq!(fs::read_to_string(copy).unwrap(), old);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupted_file_is_kept_aside_and_the_app_starts_clean() {
        for (name, bytes) in [("garbage", b"{ not json".to_vec()), ("binary", vec![0xff, 0xfe, 0x00, 0x12]), ("newer", br#"{"version":99,"subs":[]}"#.to_vec()), ("array", b"[1,2]".to_vec())] {
            let dir = tmp(name);
            fs::write(dir.join(DATA_FILE), &bytes).unwrap();
            let (d, o) = load(&dir, "20261125-100000");
            match o {
                LoadOutcome::Recovered { moved_to } => {
                    assert!(moved_to.ends_with("data.corrupt-20261125-100000.json"), "{name}");
                    assert_eq!(fs::read(moved_to).unwrap(), bytes);
                }
                other => panic!("{name}: {other:?}"),
            }
            assert!(d.subs.is_empty());
            assert!(!dir.join(DATA_FILE).exists());
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn bom_is_tolerated() {
        let dir = tmp("bom");
        fs::write(dir.join(DATA_FILE), "\u{feff}{\"version\":1,\"subs\":[]}").unwrap();
        assert_eq!(load(&dir, "x").1, LoadOutcome::Loaded);
        let _ = fs::remove_dir_all(dir);
    }
}
