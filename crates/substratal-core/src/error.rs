//! Errors, decoded from the API's `{ "error": { code, message, details } }` envelope.

use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Api(Box<ApiError>),
    #[error("couldn't reach the Substratal API: {0}")]
    Network(String),
    #[error("unexpected response from the API: {0}")]
    Decode(String),
    #[error("you're signed out")]
    SignedOut,
    #[error("this version of the app is no longer supported by the API")]
    UpdateRequired { supported: Vec<String> },
    #[error("secure storage: {0}")]
    Secrets(String),
}

impl From<ApiError> for Error {
    fn from(e: ApiError) -> Self {
        Error::Api(Box::new(e))
    }
}

impl Error {
    /// The API error code, when there is one (e.g. `version_conflict`).
    pub fn code(&self) -> Option<&str> {
        match self {
            Error::Api(e) => Some(&e.code),
            _ => None,
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api(e) => Some(e.status),
            _ => None,
        }
    }

    pub fn request_id(&self) -> Option<&str> {
        match self {
            Error::Api(e) => e.request_id.as_deref(),
            _ => None,
        }
    }
}

/// One decoded API error. `message` is for the "Details" disclosure only;
/// user-facing text comes from [`ApiError::user_message`] (COPY-AND-STATES).
#[derive(Debug, Clone, thiserror::Error)]
#[error("{status} {code}: {message}")]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub details: Value,
    pub request_id: Option<String>,
    pub retry_after: Option<Duration>,
}

#[derive(Deserialize)]
struct Envelope {
    error: Body,
}

#[derive(Deserialize)]
struct Body {
    code: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    details: Value,
}

impl ApiError {
    pub fn from_body(
        status: u16,
        body: &[u8],
        request_id: Option<String>,
        retry_after: Option<Duration>,
    ) -> Self {
        match serde_json::from_slice::<Envelope>(body) {
            Ok(env) => ApiError {
                status,
                code: env.error.code,
                message: env.error.message,
                details: env.error.details,
                request_id,
                retry_after,
            },
            Err(_) => ApiError {
                status,
                code: fallback_code(status).into(),
                message: String::from_utf8_lossy(body).chars().take(300).collect(),
                details: Value::Null,
                request_id,
                retry_after,
            },
        }
    }

    /// `details.fields[]` as `(field, code)` pairs, for inline form errors.
    pub fn field_errors(&self) -> Vec<(String, String)> {
        self.details
            .get("fields")
            .and_then(Value::as_array)
            .map(|fields| {
                fields
                    .iter()
                    .filter_map(|f| {
                        Some((
                            f.get("field")?.as_str()?.to_string(),
                            f.get("code")?.as_str()?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `details.supported` on version errors.
    pub fn supported_versions(&self) -> Vec<String> {
        self.details
            .get("supported")
            .and_then(Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The person-facing message for this error (COPY-AND-STATES → Errors).
    pub fn user_message(&self) -> String {
        let text = match self.code.as_str() {
            "invalid_credentials" => "Email or password is incorrect.",
            "invalid_mfa_code" => {
                "That code didn't work. Check your authenticator app and try again."
            }
            "user_suspended" => "This account is suspended. Contact your administrator.",
            "test_user_expired" => "This test account has expired.",
            "unauthenticated" | "session_revoked" => "Your session ended. Sign in again.",
            "forbidden" => "You don't have permission to do this.",
            "tenant_suspended" => "This organization's tenant is suspended, so changes are paused.",
            "not_found" => "This item doesn't exist or was removed.",
            "version_conflict" => "Someone else changed this while you were editing.",
            "idempotency_key_in_flight" => {
                "Still working on your last request. Try again in a second."
            }
            "entitlement_platform_managed" => {
                "Managed by Substratal — you can only turn it off or back on."
            }
            "rate_limited" => "You're doing that too often. Try again shortly.",
            "service_unavailable" => {
                "Substratal is temporarily unavailable. Try again in a moment."
            }
            "not_implemented" => "This feature isn't available yet.",
            "validation_failed" => "Some fields need attention.",
            "api_version_required" | "unsupported_api_version" | "method_not_allowed" => {
                "Something went wrong in the app. Try again, and update the app if it keeps happening."
            }
            _ => match self.status {
                401 => "Your session ended. Sign in again.",
                403 => "You don't have permission to do this.",
                404 => "This item doesn't exist or was removed.",
                409 => "This conflicts with a recent change. Reload and try again.",
                429 => "You're doing that too often. Try again shortly.",
                s if s >= 500 => "Something went wrong on our side. Try again in a moment.",
                _ => "Something went wrong. Try again.",
            },
        };
        text.to_string()
    }
}

fn fallback_code(status: u16) -> &'static str {
    match status {
        400 => "bad_request",
        401 => "unauthenticated",
        403 => "forbidden",
        404 => "not_found",
        405 => "method_not_allowed",
        409 => "conflict",
        429 => "rate_limited",
        501 => "not_implemented",
        503 => "service_unavailable",
        _ => "http_error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_envelope_and_fields() {
        let body = br#"{"error":{"code":"validation_failed","message":"bad","details":{"fields":[{"field":"email","code":"invalid"}]}}}"#;
        let e = ApiError::from_body(422, body, Some("req_1".into()), None);
        assert_eq!(e.code, "validation_failed");
        assert_eq!(e.field_errors(), vec![("email".into(), "invalid".into())]);
        assert_eq!(e.user_message(), "Some fields need attention.");
    }

    #[test]
    fn decodes_version_error() {
        let body = br#"{"error":{"code":"api_version_required","details":{"supported":["1"]},"message":"x"}}"#;
        let e = ApiError::from_body(400, body, None, None);
        assert_eq!(e.supported_versions(), vec!["1".to_string()]);
    }

    #[test]
    fn non_json_body_falls_back() {
        let e = ApiError::from_body(503, b"<html>", None, None);
        assert_eq!(e.code, "service_unavailable");
    }
}
