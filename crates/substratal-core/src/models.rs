//! Hand-written models for the operations the skeleton calls today.
//!
//! The full client will be generated from `vendor/openapi.yaml` (see
//! `planning/PLAN-management-app.md` §6.1). Until then these mirror the
//! schemas of the same names in the spec. Every struct ignores unknown
//! fields, and enums carry an `Unknown` case, because additive changes
//! (new fields, new enum values) ship within an API version
//! (CLIENT-ARCHITECTURE §4.1).

use serde::{Deserialize, Serialize};

/// `AuthSession`: the response of login, MFA verify and refresh.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthSession {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub session_id: String,
    pub user_id: String,
    #[serde(default)]
    pub email_verified: Option<bool>,
}

/// `MfaChallenge`: login succeeded but TOTP is on.
#[derive(Clone, Debug, Deserialize)]
pub struct MfaChallenge {
    pub mfa_token: String,
    #[serde(default)]
    pub methods: Vec<String>,
    pub expires_in: u64,
}

/// `POST /auth/login` answers with one of these.
#[derive(Clone, Debug)]
pub enum LoginOutcome {
    Session(AuthSession),
    MfaRequired(MfaChallenge),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Invited,
    Suspended,
    Deleted,
    #[default]
    #[serde(other)]
    Unknown,
}

impl UserStatus {
    /// Key into `status-semantics.json`.
    pub fn key(self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Invited => "invited",
            UserStatus::Suspended => "suspended",
            UserStatus::Deleted => "deleted",
            UserStatus::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Standard,
    #[default]
    #[serde(other)]
    Unknown,
}

/// `User` (the fields the shell and USR-01 use).
#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub status: UserStatus,
    #[serde(default)]
    pub mfa_enabled: bool,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub test_mode: bool,
    #[serde(default)]
    pub plan: Plan,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_login_at: Option<String>,
}

/// `PageInfo`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct PageInfo {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Any `*Page` envelope: `{ data: [...], page: { next_cursor, has_more } }`.
#[derive(Clone, Debug, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub page: PageInfo,
}

/// `GET /health`.
#[derive(Clone, Debug, Deserialize)]
pub struct Health {
    pub status: String,
}

/// A platform `Role` as listed by `GET /roles?scope=platform`.
#[derive(Clone, Debug, Deserialize)]
pub struct Role {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// `UserRoleAssignment`: one row of `GET /users/{id}/roles`.
#[derive(Clone, Debug, Deserialize)]
pub struct RoleAssignment {
    pub role_id: String,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub implicit: bool,
}

/// `ApplicationSummary` (enough to know which apps a person owns).
#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationSummary {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_enum_values_and_fields_are_tolerated() {
        let u: User = serde_json::from_str(
            r#"{"id":"usr_1","email":"a@b.c","status":"frozen","plan":"enterprise","new_field":1}"#,
        )
        .unwrap();
        assert_eq!(u.status, UserStatus::Unknown);
        assert_eq!(u.plan, Plan::Unknown);
    }

    #[test]
    fn page_envelope() {
        let p: Page<User> = serde_json::from_str(r#"{"data":[{"id":"usr_1","email":"a@b.c","status":"active"}],"page":{"next_cursor":null,"has_more":false}}"#).unwrap();
        assert_eq!(p.data.len(), 1);
        assert!(!p.page.has_more);
    }
}
