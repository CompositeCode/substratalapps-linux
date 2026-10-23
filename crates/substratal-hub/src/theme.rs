//! The theme system (THEMES.md, linux/theme/README.md): 5 light and 5 dark
//! themes as libadwaita named colours, one application-priority
//! `GtkCssProvider`, reloaded on `notify::dark`, skipped in high contrast.

use std::sync::OnceLock;

use gtk::gdk;
use serde::Deserialize;

pub const DEFAULT_LIGHT: &str = "moss-paper";
pub const DEFAULT_DARK: &str = "forest-floor";

#[derive(Clone, Debug, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub description: String,
    pub swatch: Option<String>,
}

#[derive(Deserialize)]
struct ThemeFile {
    themes: Vec<ThemeInfo>,
}

pub fn themes() -> &'static [ThemeInfo] {
    static T: OnceLock<Vec<ThemeInfo>> = OnceLock::new();
    T.get_or_init(|| {
        let f: ThemeFile = serde_json::from_str(include_str!("../../../data/spec/themes.json"))
            .expect("themes.json");
        f.themes
    })
}

pub fn themes_for(mode: &str) -> impl Iterator<Item = &'static ThemeInfo> {
    themes().iter().filter(move |t| t.mode == mode)
}

fn known(mode: &str, id: &str) -> bool {
    themes_for(mode).any(|t| t.id == id)
}

thread_local! {
    static PROVIDER: gtk::CssProvider = gtk::CssProvider::new();
}

/// Installs the providers and starts following the style manager.
pub fn init() {
    let display = gdk::Display::default().expect("a display");
    let app_css = gtk::CssProvider::new();
    app_css.load_from_resource("/com/substratalapps/Hub/style.css");
    // The theme provider sits below the app stylesheet so `style.css` can use its colours.
    PROVIDER.with(|p| {
        gtk::style_context_add_provider_for_display(
            &display,
            p,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        )
    });
    gtk::style_context_add_provider_for_display(
        &display,
        &app_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );

    let sm = adw::StyleManager::default();
    sm.connect_dark_notify(|_| apply());
    sm.connect_high_contrast_notify(|_| apply());
    apply_color_scheme();
}

/// Maps Settings.theme to AdwStyleManager:color-scheme, then loads the CSS.
pub fn apply_color_scheme() {
    let scheme = match crate::prefs::get().color_scheme.as_str() {
        "light" => adw::ColorScheme::ForceLight,
        "dark" => adw::ColorScheme::ForceDark,
        _ => adw::ColorScheme::PreferLight,
    };
    adw::StyleManager::default().set_color_scheme(scheme);
    apply();
}

/// Loads the active light or dark theme. An unknown id falls back to the
/// default for its mode without telling the person (CLIENT-ARCHITECTURE §13).
pub fn apply() {
    let sm = adw::StyleManager::default();
    PROVIDER.with(|p| {
        if sm.is_high_contrast() {
            p.load_from_string("");
            return;
        }
        let prefs = crate::prefs::get();
        let (mode, id, fallback) = if sm.is_dark() {
            ("dark", prefs.dark_theme, DEFAULT_DARK)
        } else {
            ("light", prefs.light_theme, DEFAULT_LIGHT)
        };
        let id = if known(mode, &id) {
            id
        } else {
            fallback.to_string()
        };
        p.load_from_resource(&format!("/com/substratalapps/Hub/theme/{mode}/{id}.css"));
    });
}
