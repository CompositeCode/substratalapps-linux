//! The navigation tree (INFORMATION-ARCHITECTURE §1) and its visibility
//! rules (§2). Section names, order and icon meanings are identical in every
//! Hub app; the icon column here is the Adwaita one (§5).

use crate::capabilities::{Capabilities, flag, perm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Personal,
    Administration,
    Operations,
    Developer,
}

impl Section {
    pub fn title(self) -> &'static str {
        match self {
            Section::Personal => "Personal",
            Section::Administration => "Administration",
            Section::Operations => "Operations",
            Section::Developer => "Developer",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NavItem {
    pub section: Section,
    pub title: &'static str,
    pub screen_id: &'static str,
    pub route: &'static str,
    pub icon: &'static str,
    /// Web-only items (Console, Reports, Integrations) are listed so the shared
    /// help center can describe the whole tree; Linux never shows them.
    pub web_only: bool,
    visible: fn(&Capabilities) -> bool,
}

impl NavItem {
    pub fn is_visible(&self, caps: &Capabilities) -> bool {
        !self.web_only && (self.visible)(caps)
    }

    /// The `Alt+N` shortcut for the six Personal destinations (PLATFORM-SPEC §3).
    pub fn alt_shortcut(&self) -> Option<u8> {
        PERSONAL_SHORTCUTS
            .iter()
            .position(|id| *id == self.screen_id)
            .map(|i| i as u8 + 1)
    }
}

const PERSONAL_SHORTCUTS: [&str; 6] = ["APP-01", "NTF-01", "ORG-01", "ACC-01", "BIL-01", "SET-01"];

macro_rules! item {
    ($section:ident, $title:expr, $id:expr, $route:expr, $icon:expr, $vis:expr) => {
        NavItem {
            section: Section::$section,
            title: $title,
            screen_id: $id,
            route: $route,
            icon: $icon,
            web_only: false,
            visible: $vis,
        }
    };
    (web $section:ident, $title:expr, $id:expr, $route:expr, $icon:expr) => {
        NavItem {
            section: Section::$section,
            title: $title,
            screen_id: $id,
            route: $route,
            icon: $icon,
            web_only: true,
            visible: |_| false,
        }
    };
}

pub static TREE: &[NavItem] = &[
    item!(
        Personal,
        "Home",
        "APP-01",
        "/",
        "view-app-grid-symbolic",
        |_| true
    ),
    item!(
        Personal,
        "Inbox",
        "NTF-01",
        "/inbox",
        "mail-inbox-symbolic",
        |c| c.flag(flag::NOTIFICATIONS)
    ),
    item!(
        Personal,
        "Organizations",
        "ORG-01",
        "/orgs",
        "system-users-symbolic",
        |c| c.flag(flag::ORGANIZATIONS)
    ),
    item!(
        Personal,
        "Account",
        "ACC-01",
        "/account",
        "avatar-default-symbolic",
        |_| true
    ),
    item!(
        Personal,
        "Plan & Billing",
        "BIL-01",
        "/billing",
        "wallet-symbolic",
        |c| c.flag(flag::BILLING)
    ),
    item!(
        Personal,
        "Settings",
        "SET-01",
        "/settings",
        "emblem-system-symbolic",
        |_| true
    ),
    item!(
        Administration,
        "Users",
        "USR-01",
        "/admin/users",
        "contact-new-symbolic",
        |c| c.can_list_users()
    ),
    item!(
        Administration,
        "Access",
        "ACX-05",
        "/admin/access",
        "object-select-symbolic",
        |c| c.can_manage_entitlements() || c.owns_app()
    ),
    item!(
        Administration,
        "Roles",
        "ROL-01",
        "/admin/roles",
        "security-high-symbolic",
        |c| c.can_list_users() || c.can_manage_roles()
    ),
    item!(
        Administration,
        "Audit Log",
        "AUD-01",
        "/admin/audit",
        "document-open-recent-symbolic",
        |c| c.can_view_audit()
    ),
    item!(web Administration, "Reports", "RPT-01", "/admin/reports", "x-office-spreadsheet-symbolic"),
    item!(
        Administration,
        "Send notification",
        "NTF-06",
        "/admin/notifications/new",
        "bell-symbolic",
        |c| c.can_send_notifications() && c.flag(flag::SEND_NOTIFICATIONS)
    ),
    item!(web Operations, "Overview", "OPS-01", "/ops", "speedometer-symbolic"),
    item!(web Operations, "Traffic & latency", "OPS-02", "/ops/traffic", "view-statistics-symbolic"),
    item!(web Operations, "Logs", "OPS-03", "/ops/logs", "system-search-symbolic"),
    item!(web Operations, "Security events", "SEC-01", "/ops/security", "security-medium-symbolic"),
    item!(web Operations, "Alarms", "OPS-04", "/ops/alarms", "dialog-warning-symbolic"),
    item!(web Operations, "Jobs & delivery", "OPS-05", "/ops/jobs", "alarm-symbolic"),
    item!(web Operations, "Releases", "OPS-06", "/ops/releases", "software-update-available-symbolic"),
    item!(web Operations, "Cost", "OPS-07", "/ops/cost", "money-symbolic"),
    item!(web Operations, "Configuration", "CFG-01", "/ops/config", "preferences-other-symbolic"),
    item!(
        Developer,
        "Applications",
        "APL-01",
        "/dev/apps",
        "application-x-addon-symbolic",
        |c| c.owns_app() || c.has(perm::APPLICATIONS_MANAGE)
    ),
    item!(
        Developer,
        "API Keys",
        "KEY-01",
        "/dev/keys",
        "dialog-password-symbolic",
        |c| c.owns_app() || c.has(perm::API_KEYS_MANAGE)
    ),
    item!(
        Developer,
        "Webhooks",
        "WHK-01",
        "/dev/webhooks",
        "network-transmit-receive-symbolic",
        |c| (c.owns_app() || c.has(perm::WEBHOOKS_MANAGE)) && c.flag(flag::WEBHOOKS)
    ),
    item!(web Developer, "Integrations", "INT-01", "/dev/integrations", "application-x-addon-symbolic"),
    item!(
        Developer,
        "Tenant",
        "BIL-04",
        "/dev/tenant",
        "network-server-symbolic",
        |c| !c.owned_tenants.is_empty() || c.can_manage_tenants()
    ),
];

/// Sections in sidebar order, with their visible items. A section with no
/// visible item, or whose section-level rule fails, is omitted.
pub fn visible_sections(caps: &Capabilities) -> Vec<(Section, Vec<&'static NavItem>)> {
    let mut out = Vec::new();
    for section in [
        Section::Personal,
        Section::Administration,
        Section::Developer,
    ] {
        let section_ok = match section {
            Section::Personal => true,
            Section::Administration => caps.shows_administration(),
            Section::Developer => caps.shows_developer(),
            Section::Operations => false,
        };
        if !section_ok {
            continue;
        }
        let items: Vec<_> = TREE
            .iter()
            .filter(|i| i.section == section && i.is_visible(caps))
            .collect();
        if !items.is_empty() {
            out.push((section, items));
        }
    }
    out
}

/// The nav item a screen belongs to (for highlighting the sidebar row on
/// detail screens), matched by route prefix.
pub fn item_for_route(route: &str) -> Option<&'static NavItem> {
    TREE.iter()
        .filter(|i| !i.web_only)
        .filter(|i| {
            route == i.route || (i.route != "/" && route.starts_with(&format!("{}/", i.route)))
        })
        .max_by_key(|i| i.route.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::personas;
    use crate::screens;

    fn titles(c: &Capabilities) -> Vec<&'static str> {
        visible_sections(c)
            .into_iter()
            .flat_map(|(_, items)| items.into_iter().map(|i| i.title))
            .collect()
    }

    #[test]
    fn every_item_points_at_a_real_screen_and_route() {
        for item in TREE {
            let s = screens::get(item.screen_id)
                .unwrap_or_else(|| panic!("{} missing", item.screen_id));
            assert_eq!(s.route, item.route, "{}", item.screen_id);
            assert_eq!(item.web_only, !s.on_linux(), "{}", item.screen_id);
        }
    }

    #[test]
    fn end_user_sees_personal_only() {
        let c = personas::end_user();
        assert_eq!(
            titles(&c),
            [
                "Home",
                "Inbox",
                "Organizations",
                "Account",
                "Plan & Billing",
                "Settings"
            ]
        );
    }

    #[test]
    fn app_owner_gets_confined_access_and_developer() {
        let t = titles(&personas::app_owner());
        assert!(t.contains(&"Access") && !t.contains(&"Users") && !t.contains(&"Audit Log"));
        assert!(
            t.contains(&"Applications")
                && t.contains(&"API Keys")
                && t.contains(&"Webhooks")
                && t.contains(&"Tenant")
        );
    }

    #[test]
    fn support_and_billing_admin() {
        let s = titles(&personas::support());
        assert!(
            s.contains(&"Users")
                && s.contains(&"Access")
                && s.contains(&"Audit Log")
                && s.contains(&"Tenant")
        );
        assert!(!s.contains(&"Send notification") && !s.contains(&"API Keys"));
        let b = titles(&personas::billing_admin());
        assert!(b.contains(&"Users") && b.contains(&"Roles") && b.contains(&"Audit Log"));
        assert!(!b.contains(&"Access") && !b.contains(&"Applications"));
    }

    #[test]
    fn superadmin_sees_everything_but_web_only() {
        let t = titles(&personas::superadmin());
        assert_eq!(t.len(), TREE.iter().filter(|i| !i.web_only).count());
        assert!(!t.contains(&"Overview") && !t.contains(&"Reports"));
    }

    #[test]
    fn flags_hide_items() {
        let mut c = personas::superadmin();
        c.disable_flag(flag::NOTIFICATIONS);
        c.disable_flag(flag::WEBHOOKS);
        let t = titles(&c);
        assert!(!t.contains(&"Inbox") && !t.contains(&"Webhooks"));
    }

    #[test]
    fn route_to_item() {
        assert_eq!(
            item_for_route("/admin/users/usr_1").unwrap().screen_id,
            "USR-01"
        );
        assert_eq!(item_for_route("/").unwrap().screen_id, "APP-01");
        assert_eq!(item_for_route("/apps/past").map(|i| i.screen_id), None);
    }
}
