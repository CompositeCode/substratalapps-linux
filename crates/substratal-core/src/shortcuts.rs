//! Keyboard shortcuts (linux/PLATFORM-SPEC.md §3).
//!
//! One table drives the app's `GtkShortcutController`s, its Keyboard
//! Shortcuts dialog, and the help center's shortcuts page.

#[derive(Clone, Copy, Debug)]
pub struct Shortcut {
    /// GTK accelerator syntax, e.g. `<Control>k`.
    pub accels: &'static [&'static str],
    /// The `GAction` it triggers (`app.` or `win.`), if it maps to one.
    pub action: Option<&'static str>,
    pub label: &'static str,
    pub group: &'static str,
}

pub static SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        accels: &["<Control>k"],
        action: Some("win.command-palette"),
        label: "Command palette",
        group: "General",
    },
    Shortcut {
        accels: &["<Control>f"],
        action: Some("win.search"),
        label: "Search",
        group: "General",
    },
    Shortcut {
        accels: &["F1"],
        action: Some("win.help"),
        label: "Help for this screen",
        group: "General",
    },
    Shortcut {
        accels: &["<Control>comma"],
        action: Some("win.preferences"),
        label: "Preferences",
        group: "General",
    },
    Shortcut {
        accels: &["<Control>question"],
        action: Some("win.show-shortcuts"),
        label: "Keyboard shortcuts",
        group: "General",
    },
    Shortcut {
        accels: &["<Control>w"],
        action: Some("window.close"),
        label: "Close window",
        group: "General",
    },
    Shortcut {
        accels: &["<Control>q"],
        action: Some("app.quit"),
        label: "Quit",
        group: "General",
    },
    Shortcut {
        accels: &["<Alt>1"],
        action: Some("win.go::APP-01"),
        label: "Home",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Alt>2"],
        action: Some("win.go::NTF-01"),
        label: "Inbox",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Alt>3"],
        action: Some("win.go::ORG-01"),
        label: "Organizations",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Alt>4"],
        action: Some("win.go::ACC-01"),
        label: "Account",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Alt>5"],
        action: Some("win.go::BIL-01"),
        label: "Plan & Billing",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Alt>6"],
        action: Some("win.go::SET-01"),
        label: "Settings",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Alt>Left"],
        action: Some("win.back"),
        label: "Back",
        group: "Navigation",
    },
    Shortcut {
        accels: &["F9"],
        action: Some("win.toggle-sidebar"),
        label: "Show or hide the sidebar",
        group: "Navigation",
    },
    Shortcut {
        accels: &["<Control>n"],
        action: Some("win.new"),
        label: "New (in the current list)",
        group: "Editing",
    },
    Shortcut {
        accels: &["<Control>r", "F5"],
        action: Some("win.refresh"),
        label: "Refresh",
        group: "Editing",
    },
    Shortcut {
        accels: &["<Control>z"],
        action: Some("win.undo"),
        label: "Undo an access change",
        group: "Editing",
    },
    Shortcut {
        accels: &["Delete"],
        action: Some("win.delete"),
        label: "Delete or turn off the selection (asks first)",
        group: "Editing",
    },
    Shortcut {
        accels: &["<Control><Shift>u"],
        action: Some("win.mark-all-read"),
        label: "Mark all as read (Inbox)",
        group: "Editing",
    },
];

/// Human-readable form of an accelerator, e.g. `<Control><Shift>u` → `Ctrl+Shift+U`.
pub fn display(accel: &str) -> String {
    let mut parts = Vec::new();
    let mut rest = accel;
    while let Some(stripped) = rest.strip_prefix('<') {
        let end = stripped.find('>').unwrap_or(stripped.len());
        parts.push(match &stripped[..end] {
            "Control" | "Primary" => "Ctrl".to_string(),
            other => other.to_string(),
        });
        rest = &stripped[(end + 1).min(stripped.len())..];
    }
    let key = match rest {
        "comma" => ",".to_string(),
        "question" => "?".to_string(),
        "Left" => "←".to_string(),
        k if k.len() == 1 => k.to_uppercase(),
        k => k.to_string(),
    };
    parts.push(key);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_accelerators() {
        assert_eq!(display("<Control><Shift>u"), "Ctrl+Shift+U");
        assert_eq!(display("<Control>question"), "Ctrl+?");
        assert_eq!(display("F5"), "F5");
        assert_eq!(display("<Alt>Left"), "Alt+←");
    }

    #[test]
    fn no_accelerator_is_bound_twice() {
        let mut seen = std::collections::HashSet::new();
        for s in SHORTCUTS {
            for a in s.accels {
                assert!(seen.insert(*a), "{a} bound twice");
            }
        }
    }
}
