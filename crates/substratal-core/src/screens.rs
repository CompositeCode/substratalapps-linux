//! The screen registry: every screen in the specs, embedded at build time.
//!
//! `data/spec/screens.json` is generated from the specs package by
//! `scripts/sync-specs.py` and never edited by hand. The app builds its
//! scaffold pages from it, the router matches routes against it, and the
//! help center generates one page per screen from it, so all three agree.

use std::sync::OnceLock;

use serde::Deserialize;

static RAW: &str = include_str!("../../../data/spec/screens.json");

#[derive(Clone, Debug, Deserialize)]
pub struct Element {
    pub name: String,
    pub kind: String,
    /// The GTK/libadwaita control the Linux spec names (empty for web-only screens).
    #[serde(default)]
    pub control: String,
    #[serde(default)]
    pub data: String,
    #[serde(default)]
    pub rules: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Action {
    pub action: String,
    #[serde(default)]
    pub call: String,
    #[serde(default)]
    pub result: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Screen {
    pub id: String,
    pub title: String,
    pub section_heading: Option<String>,
    pub domain: String,
    pub route: String,
    pub archetype: String,
    pub phase: String,
    pub who: String,
    pub purpose: String,
    pub loads: String,
    pub states: String,
    pub notes: String,
    pub entry_points: String,
    pub container: String,
    pub elements: Vec<Element>,
    pub actions: Vec<Action>,
    pub platforms: Vec<String>,
}

impl Screen {
    /// False for the web-only Console screens (OPEN-QUESTIONS #27).
    pub fn on_linux(&self) -> bool {
        self.platforms.iter().any(|p| p == "all" || p == "linux")
    }

    /// Screens outside the app shell (AUTH-*).
    pub fn is_auth(&self) -> bool {
        self.archetype == "auth"
    }

    /// The route with `:params` substituted, if all are given.
    pub fn path_with(&self, params: &[(&str, &str)]) -> String {
        let mut out = self.route.clone();
        for (k, v) in params {
            out = out.replace(&format!(":{k}"), v);
        }
        out
    }

    pub fn help_url(&self) -> String {
        crate::config::help_url(&self.id)
    }
}

#[derive(Debug, Deserialize)]
struct Registry {
    commit: String,
    screens: Vec<Screen>,
}

fn registry() -> &'static Registry {
    static REG: OnceLock<Registry> = OnceLock::new();
    REG.get_or_init(|| serde_json::from_str(RAW).expect("data/spec/screens.json is valid"))
}

/// Every screen, in spec order.
pub fn all() -> &'static [Screen] {
    &registry().screens
}

/// The specs commit the registry was generated from.
pub fn spec_commit() -> &'static str {
    &registry().commit
}

pub fn get(id: &str) -> Option<&'static Screen> {
    all().iter().find(|s| s.id.eq_ignore_ascii_case(id))
}

/// A matched route: the screen and its `:params`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteMatch {
    pub screen_id: String,
    pub params: Vec<(String, String)>,
}

/// Matches a path (with or without query) against every route, preferring
/// the most specific (fewest parameters). Only routes of screens available
/// on Linux match, unless `include_web` is set.
pub fn match_route(path: &str, include_web: bool) -> Option<RouteMatch> {
    let path = path.split(['?', '#']).next().unwrap_or("");
    let want: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut best: Option<(usize, RouteMatch)> = None;
    for s in all() {
        if !s.route.starts_with('/') || (!include_web && !s.on_linux()) {
            continue;
        }
        let pattern: Vec<&str> = s
            .route
            .trim_matches('/')
            .split('/')
            .filter(|x| !x.is_empty())
            .collect();
        if pattern.len() != want.len() {
            continue;
        }
        let mut params = Vec::new();
        let ok = pattern.iter().zip(&want).all(|(p, w)| {
            if let Some(name) = p.strip_prefix(':') {
                params.push((name.to_string(), (*w).to_string()));
                true
            } else {
                p == w
            }
        });
        if ok {
            let score = params.len();
            if best.as_ref().is_none_or(|(b, _)| score < *b) {
                best = Some((
                    score,
                    RouteMatch {
                        screen_id: s.id.clone(),
                        params,
                    },
                ));
            }
        }
    }
    best.map(|(_, m)| m)
}

/// Parses a `substratal://` deep link (or a dashboard URL) into a route.
/// Deep links are untrusted input (CLIENT-ARCHITECTURE §11): only known
/// routes are accepted.
pub fn match_link(link: &str) -> Option<RouteMatch> {
    let rest = link
        .strip_prefix("substratal://")
        .map(|r| format!("/{r}"))
        .or_else(|| {
            link.strip_prefix(crate::config::DASHBOARD)
                .map(str::to_string)
        })?;
    match_route(&rest, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_every_screen() {
        assert_eq!(all().iter().filter(|s| s.on_linux()).count(), 75);
        assert!(
            all()
                .iter()
                .filter(|s| !s.on_linux())
                .all(|s| s.phase.contains("web only"))
        );
        let mut ids: Vec<_> = all().iter().map(|s| s.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all().len(), "screen ids are unique");
    }

    #[test]
    fn routes_match_most_specific() {
        assert_eq!(
            match_route("/admin/users", false).unwrap().screen_id,
            "USR-01"
        );
        let m = match_route("/admin/users/usr_123", false).unwrap();
        assert_eq!(m.screen_id, "USR-03");
        assert_eq!(
            m.params,
            vec![("userId".to_string(), "usr_123".to_string())]
        );
        assert_eq!(
            match_route("/apps/past", false).unwrap().screen_id,
            "APP-05"
        );
        assert_eq!(match_route("/", false).unwrap().screen_id, "APP-01");
        assert_eq!(
            match_route("/admin/users?status=active", false)
                .unwrap()
                .screen_id,
            "USR-01"
        );
    }

    #[test]
    fn web_only_routes_need_opt_in() {
        assert!(match_route("/ops/logs", false).is_none());
        assert_eq!(match_route("/ops/logs", true).unwrap().screen_id, "OPS-03");
    }

    #[test]
    fn deep_links() {
        assert_eq!(
            match_link("substratal://admin/roles").unwrap().screen_id,
            "ROL-01"
        );
        assert_eq!(
            match_link("https://dashboard.substratalapps.com/billing")
                .unwrap()
                .screen_id,
            "BIL-01"
        );
        assert!(match_link("https://evil.example/admin/roles").is_none());
        assert!(match_link("substratal://nope/nothing").is_none());
    }
}
