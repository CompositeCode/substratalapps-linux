//! Device-local preferences: theme choice and window size.
//!
//! The theme choice is applied before the first window paints, then
//! reconciled with the account's settings once signed in (CLIENT-ARCHITECTURE
//! §13; OPEN-QUESTIONS #22 covers where the server stores the light/dark
//! theme ids). Nothing secret is ever written here.

use std::cell::RefCell;
use std::path::PathBuf;

use gtk::glib;
use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefs {
    /// `light`, `dark` or `system` (Settings.theme).
    pub color_scheme: String,
    pub light_theme: String,
    pub dark_theme: String,
    pub width: i32,
    pub height: i32,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            color_scheme: "system".into(),
            light_theme: crate::theme::DEFAULT_LIGHT.into(),
            dark_theme: crate::theme::DEFAULT_DARK.into(),
            width: 1280,
            height: 800,
        }
    }
}

fn path() -> PathBuf {
    glib::user_config_dir()
        .join(crate::APP_ID)
        .join("preferences.json")
}

thread_local! {
    static PREFS: RefCell<Prefs> = RefCell::new(load());
}

fn load() -> Prefs {
    let Ok(text) = std::fs::read_to_string(path()) else {
        return Prefs::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Prefs::default();
    };
    let d = Prefs::default();
    let s = |k: &str, def: &str| v.get(k).and_then(Value::as_str).unwrap_or(def).to_string();
    let i = |k: &str, def: i32| {
        v.get(k)
            .and_then(Value::as_i64)
            .map(|n| n as i32)
            .unwrap_or(def)
    };
    Prefs {
        color_scheme: s("color_scheme", &d.color_scheme),
        light_theme: s("light_theme", &d.light_theme),
        dark_theme: s("dark_theme", &d.dark_theme),
        width: i("width", d.width),
        height: i("height", d.height),
    }
}

pub fn get() -> Prefs {
    PREFS.with(|p| p.borrow().clone())
}

pub fn update(f: impl FnOnce(&mut Prefs)) {
    PREFS.with(|p| {
        let mut p = p.borrow_mut();
        f(&mut p);
        let v = json!({
            "color_scheme": p.color_scheme,
            "light_theme": p.light_theme,
            "dark_theme": p.dark_theme,
            "width": p.width,
            "height": p.height,
        });
        let file = path();
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&file, serde_json::to_string_pretty(&v).unwrap()) {
            eprintln!("substratal: couldn't save preferences: {e}");
        }
    });
}
