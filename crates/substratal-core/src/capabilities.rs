//! Capabilities: what the signed-in person may see and do
//! (CLIENT-ARCHITECTURE §5, INFORMATION-ARCHITECTURE §2).
//!
//! Navigation and every action read only from [`Capabilities`]; the server
//! stays the authority, and a `403` triggers a background refresh.

use std::collections::BTreeSet;

use crate::models::Plan;

/// The platform permission catalog (Roles & Permissions → Platform permission catalog).
pub mod perm {
    pub const USERS_LIST: &str = "users.list";
    pub const USERS_MANAGE: &str = "users.manage";
    pub const ENTITLEMENTS_MANAGE: &str = "entitlements.manage";
    pub const APPLICATIONS_MANAGE: &str = "applications.manage";
    pub const ROLES_MANAGE: &str = "roles.manage";
    pub const ORGANIZATIONS_MANAGE: &str = "organizations.manage";
    pub const BILLING_MANAGE: &str = "billing.manage";
    pub const BILLING_REFUND: &str = "billing.refund";
    pub const AUDIT_VIEW: &str = "audit.view";
    pub const WEBHOOKS_MANAGE: &str = "webhooks.manage";
    pub const API_KEYS_MANAGE: &str = "api_keys.manage";
    pub const NOTIFICATIONS_SEND: &str = "notifications.send";
    pub const TENANTS_MANAGE: &str = "tenants.manage";

    pub const ALL: [&str; 13] = [
        USERS_LIST,
        USERS_MANAGE,
        ENTITLEMENTS_MANAGE,
        APPLICATIONS_MANAGE,
        ROLES_MANAGE,
        ORGANIZATIONS_MANAGE,
        BILLING_MANAGE,
        BILLING_REFUND,
        AUDIT_VIEW,
        WEBHOOKS_MANAGE,
        API_KEYS_MANAGE,
        NOTIFICATIONS_SEND,
        TENANTS_MANAGE,
    ];
}

/// Client feature flags (CLIENT-ARCHITECTURE §10). Compile-time defaults
/// until the API serves remote config; a `404`/`501` from a gated endpoint
/// turns a flag off for the session.
pub mod flag {
    pub const NOTIFICATIONS: &str = "notifications";
    pub const ORGANIZATIONS: &str = "organizations";
    pub const BILLING: &str = "billing";
    pub const WEBHOOKS: &str = "webhooks";
    pub const SEND_NOTIFICATIONS: &str = "send_notifications";
    pub const PUSH_CREDENTIALS: &str = "push_credentials";
    pub const SCIM: &str = "scim";
    pub const SSO: &str = "sso";
    pub const PASSKEYS: &str = "passkeys";
    pub const AUDIT_EXPORT: &str = "audit_export";
    pub const APP_REGISTRATION_SELF_SERVICE: &str = "app_registration_self_service";
    /// Linux has no push provider, so `push` stays off; notifications arrive by polling.
    pub const PUSH: &str = "push";

    /// Flags on by default in this build.
    pub const DEFAULT_ON: &[&str] = &[
        NOTIFICATIONS,
        ORGANIZATIONS,
        BILLING,
        WEBHOOKS,
        SEND_NOTIFICATIONS,
        PUSH_CREDENTIALS,
        SCIM,
        AUDIT_EXPORT,
        APP_REGISTRATION_SELF_SERVICE,
    ];
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub signed_in: bool,
    pub permissions: BTreeSet<String>,
    pub owned_apps: Vec<String>,
    pub owned_tenants: Vec<String>,
    pub admin_of_orgs: Vec<String>,
    pub plan: Option<Plan>,
    pub mfa_enabled: bool,
    pub flags: BTreeSet<String>,
}

impl Capabilities {
    /// A signed-in person with no permissions and the default flags.
    pub fn member() -> Self {
        Capabilities {
            signed_in: true,
            flags: flag::DEFAULT_ON.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    pub fn with_permissions<I: IntoIterator<Item = S>, S: Into<String>>(
        mut self,
        perms: I,
    ) -> Self {
        self.permissions.extend(perms.into_iter().map(Into::into));
        self
    }

    pub fn has(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }

    pub fn disable_flag(&mut self, name: &str) {
        self.flags.remove(name);
    }

    pub fn owns_app(&self) -> bool {
        !self.owned_apps.is_empty()
    }

    pub fn can_list_users(&self) -> bool {
        self.has(perm::USERS_LIST)
    }
    pub fn can_manage_users(&self) -> bool {
        self.has(perm::USERS_MANAGE)
    }
    pub fn can_manage_entitlements(&self) -> bool {
        self.has(perm::ENTITLEMENTS_MANAGE)
    }
    pub fn can_manage_roles(&self) -> bool {
        self.has(perm::ROLES_MANAGE)
    }
    pub fn can_view_audit(&self) -> bool {
        self.has(perm::AUDIT_VIEW)
    }
    pub fn can_send_notifications(&self) -> bool {
        self.has(perm::NOTIFICATIONS_SEND)
    }
    pub fn can_manage_tenants(&self) -> bool {
        self.has(perm::TENANTS_MANAGE)
    }
    pub fn can_manage_billing(&self) -> bool {
        self.has(perm::BILLING_MANAGE)
    }

    /// INFORMATION-ARCHITECTURE §2: Administration section.
    pub fn shows_administration(&self) -> bool {
        [
            perm::USERS_LIST,
            perm::ENTITLEMENTS_MANAGE,
            perm::ROLES_MANAGE,
            perm::AUDIT_VIEW,
            perm::NOTIFICATIONS_SEND,
        ]
        .iter()
        .any(|p| self.has(p))
            || self.owns_app()
    }

    /// INFORMATION-ARCHITECTURE §2: Developer section.
    pub fn shows_developer(&self) -> bool {
        self.owns_app()
            || !self.owned_tenants.is_empty()
            || [
                perm::APPLICATIONS_MANAGE,
                perm::API_KEYS_MANAGE,
                perm::WEBHOOKS_MANAGE,
                perm::TENANTS_MANAGE,
            ]
            .iter()
            .any(|p| self.has(p))
    }

    /// Anyone holding a platform permission gets the admin protections
    /// (idle lock, memory-only admin data).
    pub fn is_staff(&self) -> bool {
        !self.permissions.is_empty()
    }
}

/// Personas from PRODUCT-SPEC §1, as Capabilities, for tests and previews.
pub mod personas {
    use super::*;

    pub fn end_user() -> Capabilities {
        Capabilities::member()
    }
    pub fn app_owner() -> Capabilities {
        let mut c = Capabilities::member();
        c.owned_apps = vec!["app_demo".into()];
        c.owned_tenants = vec!["tnt_demo".into()];
        c
    }
    pub fn support() -> Capabilities {
        Capabilities::member().with_permissions([
            perm::USERS_LIST,
            perm::ENTITLEMENTS_MANAGE,
            perm::AUDIT_VIEW,
            perm::TENANTS_MANAGE,
            perm::BILLING_MANAGE,
        ])
    }
    pub fn billing_admin() -> Capabilities {
        Capabilities::member().with_permissions([
            perm::BILLING_MANAGE,
            perm::BILLING_REFUND,
            perm::AUDIT_VIEW,
            perm::USERS_LIST,
        ])
    }
    pub fn superadmin() -> Capabilities {
        Capabilities::member().with_permissions(perm::ALL)
    }

    pub fn all() -> Vec<(&'static str, Capabilities)> {
        vec![
            ("End user", end_user()),
            ("App owner", app_owner()),
            ("Support", support()),
            ("Billing admin", billing_admin()),
            ("Superadmin", superadmin()),
        ]
    }
}
