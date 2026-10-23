//! Environments and well-known URLs.

use url::Url;

/// Production API base URL: the host root. There is no `/v1` (see [`crate::version`]).
pub const PRODUCTION_API: &str = "https://api.substratalapps.com";
/// The API repo's local devserver (`make up && make run`).
pub const LOCAL_API: &str = "http://localhost:8080";
/// Root of the Substratal Hub help center: the one documentation site for
/// every Hub app, built from `help/` in the specs repo
/// (CompositeCode/substratalapps-applications-specs).
pub const HELP_BASE: &str =
    "https://compositecode.github.io/substratalapps-applications-specs/help/";
/// The Platform API's own specification and reference.
pub const API_DOCS: &str = "https://compositecode.github.io/substratalapps.com/";
/// The web dashboard, where emailed links and Stripe returns land.
pub const DASHBOARD: &str = "https://dashboard.substratalapps.com";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Environment {
    Production,
    Local,
    Custom(Url),
}

impl Environment {
    /// Reads `SUBSTRATAL_API` (`production`, `local`, or a URL). Defaults to production.
    pub fn from_env() -> Self {
        match std::env::var("SUBSTRATAL_API") {
            Ok(v) => Self::parse(&v),
            Err(_) => Environment::Production,
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim() {
            "" | "production" | "prod" => Environment::Production,
            "local" | "devserver" => Environment::Local,
            other => match Url::parse(other) {
                Ok(u) if u.as_str().trim_end_matches('/') == PRODUCTION_API => {
                    Environment::Production
                }
                Ok(u) => Environment::Custom(u),
                Err(_) => Environment::Production,
            },
        }
    }

    pub fn base_url(&self) -> Url {
        let raw = match self {
            Environment::Production => PRODUCTION_API,
            Environment::Local => LOCAL_API,
            Environment::Custom(u) => u.as_str(),
        };
        let mut url = Url::parse(raw).expect("valid base URL");
        // Paths are joined onto the root; make sure there is exactly one slash.
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        url
    }

    /// Short label shown in the UI for anything but production.
    pub fn label(&self) -> Option<String> {
        match self {
            Environment::Production => None,
            Environment::Local => Some("Local devserver".into()),
            Environment::Custom(u) => Some(u.host_str().unwrap_or("custom").to_string()),
        }
    }

    /// Key used to keep credentials for different environments apart.
    pub fn key(&self) -> String {
        match self {
            Environment::Production => "production".into(),
            Environment::Local => "local".into(),
            Environment::Custom(u) => u.as_str().trim_end_matches('/').to_string(),
        }
    }
}

/// Help center page for a screen, e.g. `…/help/screens/usr-01.html`. Every Hub app
/// (Linux, macOS, Windows, iOS, Android, Web) shares these screen IDs, so the
/// same URL works as a help link from any of them.
pub fn help_url(screen_id: &str) -> String {
    format!("{HELP_BASE}screens/{}.html", screen_id.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_is_host_root_without_version() {
        let url = Environment::Production.base_url();
        assert_eq!(url.as_str(), "https://api.substratalapps.com/");
        assert_eq!(
            url.join("users").unwrap().as_str(),
            "https://api.substratalapps.com/users"
        );
        assert!(!url.as_str().contains("/v1"));
    }

    #[test]
    fn parse_values() {
        assert_eq!(Environment::parse("local"), Environment::Local);
        assert_eq!(
            Environment::parse("https://api.substratalapps.com/"),
            Environment::Production
        );
        assert!(matches!(
            Environment::parse("https://staging.example.com"),
            Environment::Custom(_)
        ));
    }

    #[test]
    fn help_urls_are_lowercase() {
        assert_eq!(
            help_url("USR-01"),
            "https://compositecode.github.io/substratalapps-applications-specs/help/screens/usr-01.html"
        );
    }
}
