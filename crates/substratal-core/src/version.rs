//! API versioning (CLIENT-ARCHITECTURE §4.1, API Conventions → Versioning).
//!
//! The Platform API's version is a **request header, not part of the URL**.
//! Since API v1.7.0 (`openapi.yaml` 1.0.0-draft.4) there is no `/v1` path
//! prefix: every endpoint lives at the host root (`https://api.substratalapps.com/users`)
//! and `/v1/...` returns `404 not_found`. Each request names its version in
//! `Substratal-Version`. Each build of this app is written against exactly
//! one version, a compile-time constant that is never negotiated or read
//! from remote config. Moving to a new version is a code change shipped as
//! an app update.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The API version this build is written against.
pub const API_VERSION: &str = "1";

/// The request (and echoed response) header that carries [`API_VERSION`].
pub const VERSION_HEADER: &str = "Substratal-Version";

/// Whether the server still serves this build's API version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VersionStatus {
    /// Served and not deprecated.
    #[default]
    Current,
    /// Responses carry `Deprecation` (RFC 9745) and usually `Sunset` (RFC 8594).
    /// The UI shows GLB-03 "Update available" once a day.
    Deprecated {
        deprecated_at: Option<SystemTime>,
        sunset: Option<SystemTime>,
    },
    /// `400 unsupported_api_version`: the version is past its sunset (or the
    /// environment is wrong). The UI stops API traffic and shows GLB-03
    /// "Update required".
    Unsupported { supported: Vec<String> },
}

impl VersionStatus {
    pub fn is_unsupported(&self) -> bool {
        matches!(self, VersionStatus::Unsupported { .. })
    }
}

/// Parses a `Deprecation` header. RFC 9745 sends a structured-field date,
/// `@<unix seconds>`. Earlier drafts sent `true` or an HTTP-date; accept
/// those too, since any value means "deprecated".
pub fn parse_deprecation(value: &str) -> Option<SystemTime> {
    let v = value.trim();
    if let Some(secs) = v.strip_prefix('@') {
        return secs
            .parse::<u64>()
            .ok()
            .map(|s| UNIX_EPOCH + Duration::from_secs(s));
    }
    httpdate::parse_http_date(v).ok()
}

/// Parses a `Sunset` header (an HTTP-date, RFC 8594).
pub fn parse_sunset(value: &str) -> Option<SystemTime> {
    httpdate::parse_http_date(value.trim()).ok()
}

/// Formats a time as an HTTP-date for display and logs.
pub fn fmt_date(t: SystemTime) -> String {
    httpdate::fmt_http_date(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deprecation_structured_field_date() {
        let t = parse_deprecation("@1688169599").unwrap();
        assert_eq!(t, UNIX_EPOCH + Duration::from_secs(1_688_169_599));
    }

    #[test]
    fn deprecation_http_date_and_garbage() {
        assert!(parse_deprecation("Sun, 30 Jun 2024 23:59:59 GMT").is_some());
        assert!(parse_deprecation("true").is_none());
    }

    #[test]
    fn sunset_http_date() {
        let t = parse_sunset("Wed, 01 Jul 2026 00:00:00 GMT").unwrap();
        assert_eq!(fmt_date(t), "Wed, 01 Jul 2026 00:00:00 GMT");
    }
}
