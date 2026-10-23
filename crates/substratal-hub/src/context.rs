//! Shared app state: the API client, the person and their Capabilities.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use substratal_core::capabilities::Capabilities;
use substratal_core::models::User;
use substratal_core::secrets::SecretStore;
use substratal_core::{ApiClient, Environment};

pub struct AppContext {
    pub api: Arc<ApiClient>,
    pub caps: RefCell<Capabilities>,
    pub me: RefCell<Option<User>>,
    /// Developer preview persona (`SUBSTRATAL_PREVIEW`), if any.
    pub preview: Option<String>,
}

impl AppContext {
    pub fn new() -> Rc<Self> {
        let env = Environment::from_env();
        let preview = std::env::var("SUBSTRATAL_PREVIEW")
            .ok()
            .filter(|s| !s.is_empty());
        let secrets = if preview.is_some() {
            SecretStore::memory()
        } else {
            crate::runtime::block_on(SecretStore::system())
        };
        let api =
            ApiClient::new(env, secrets, &user_agent(), &accept_language()).expect("HTTP client");
        Rc::new(AppContext {
            api: Arc::new(api),
            caps: RefCell::new(Capabilities::default()),
            me: RefCell::new(None),
            preview,
        })
    }

    pub fn is_preview(&self) -> bool {
        self.preview.is_some()
    }
}

/// `Substratal-Linux/{version} ({distro})`, shown to people in their Sessions list.
fn user_agent() -> String {
    let distro = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.strip_prefix("PRETTY_NAME=")
                    .map(|v| v.trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "Linux".into());
    format!("Substratal-Linux/{} ({distro})", crate::VERSION)
}

fn accept_language() -> String {
    gtk::glib::language_names()
        .iter()
        .map(|l| l.as_str())
        .find(|l| *l != "C" && !l.contains('.'))
        .map(|l| l.replace('_', "-"))
        .unwrap_or_else(|| "en-US".into())
}
