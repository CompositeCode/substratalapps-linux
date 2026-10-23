//! Small shared widgets (COMPONENTS.md).

use std::collections::HashMap;
use std::sync::OnceLock;

use adw::prelude::*;
use serde_json::Value;

/// A full-page `AdwStatusPage`.
pub fn status_page(icon: &str, title: &str, description: &str) -> gtk::Widget {
    adw::StatusPage::builder()
        .icon_name(icon)
        .title(title)
        .description(glib_escape(description))
        .vexpand(true)
        .build()
        .upcast()
}

/// Escapes text for Pango markup (status page descriptions use markup).
pub fn glib_escape(s: &str) -> String {
    gtk::glib::markup_escape_text(s).to_string()
}

struct StatusInfo {
    label: String,
    role: String,
    icon: Option<String>,
}

fn statuses() -> &'static HashMap<String, StatusInfo> {
    static S: OnceLock<HashMap<String, StatusInfo>> = OnceLock::new();
    S.get_or_init(|| {
        let v: Value =
            serde_json::from_str(include_str!("../../../data/spec/status-semantics.json"))
                .unwrap_or(Value::Null);
        v.get("statuses")
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .map(|(k, s)| {
                        (
                            k.clone(),
                            StatusInfo {
                                label: s
                                    .get("label")
                                    .and_then(Value::as_str)
                                    .unwrap_or(k)
                                    .to_string(),
                                role: s
                                    .get("role")
                                    .and_then(Value::as_str)
                                    .unwrap_or("neutral")
                                    .to_string(),
                                icon: s
                                    .pointer("/icon/adwaita")
                                    .and_then(Value::as_str)
                                    .map(str::to_string),
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// StatusBadge: icon + label, never colour alone (status-semantics.json).
pub fn status_badge(status: &str) -> gtk::Widget {
    let (label, role, icon) = match statuses().get(status) {
        Some(s) => (s.label.clone(), s.role.clone(), s.icon.clone()),
        None => (status.to_string(), "neutral".to_string(), None),
    };
    let b = gtk::Box::builder()
        .spacing(4)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Center)
        .build();
    b.add_css_class("status-badge");
    b.add_css_class(&role);
    if let Some(icon) = icon {
        b.append(&gtk::Image::from_icon_name(&icon));
    }
    b.append(&gtk::Label::new(Some(&label)));
    b.update_property(&[gtk::accessible::Property::Label(&format!(
        "Status: {label}"
    ))]);
    b.upcast()
}

/// A monospace chip showing a screen or object id.
pub fn id_chip(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.add_css_class("screen-id");
    l.set_selectable(true);
    l
}
