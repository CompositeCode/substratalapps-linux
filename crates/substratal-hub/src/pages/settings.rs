//! SET-01 Settings: the Appearance group (THEMES.md, PLATFORM-SPEC §4c)
//! is live; the rest of the screen comes from the spec scaffold.

use std::rc::Rc;

use adw::prelude::*;
use substratal_core::screens::Screen;

use crate::shell::Shell;
use crate::{prefs, theme};

pub fn page(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    let appearance = adw::PreferencesGroup::builder()
        .title("Appearance")
        .description(
            "Applies on this device now and follows you to your other devices once signed in.",
        )
        .build();

    // Style: Match system / Light / Dark (Settings.theme).
    let style_row = adw::ActionRow::builder().title("Style").build();
    let linked = gtk::Box::builder().valign(gtk::Align::Center).build();
    linked.add_css_class("linked");
    let current = prefs::get().color_scheme;
    let mut group_leader: Option<gtk::ToggleButton> = None;
    for (id, label) in [
        ("system", "Match system"),
        ("light", "Light"),
        ("dark", "Dark"),
    ] {
        let b = gtk::ToggleButton::builder()
            .label(label)
            .active(current == id)
            .build();
        if let Some(lead) = &group_leader {
            b.set_group(Some(lead));
        } else {
            group_leader = Some(b.clone());
        }
        b.connect_toggled(move |b| {
            if b.is_active() {
                prefs::update(|p| p.color_scheme = id.to_string());
                theme::apply_color_scheme();
            }
        });
        linked.append(&b);
    }
    style_row.add_suffix(&linked);
    appearance.add(&style_row);

    for (mode, title) in [("light", "Light theme"), ("dark", "Dark theme")] {
        appearance.add(&swatch_row(mode, title));
    }

    let column = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_start(16)
        .margin_end(16)
        .build();
    column.append(&appearance);
    let clamp = adw::Clamp::builder()
        .maximum_size(760)
        .child(&column)
        .build();
    let outer = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    outer.append(&clamp);
    // The rest of SET-01 (locale, notifications, privacy…) from the spec.
    outer.append(&super::scaffold_content(shell, screen));
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&outer)
        .build()
        .upcast()
}

/// A `GtkFlowBox` of swatch buttons for one mode.
fn swatch_row(mode: &'static str, title: &str) -> gtk::Widget {
    let selected = match mode {
        "light" => prefs::get().light_theme,
        _ => prefs::get().dark_theme,
    };
    let row = adw::PreferencesRow::builder().activatable(false).build();
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let heading = gtk::Label::builder().label(title).xalign(0.0).build();
    b.append(&heading);
    let name = gtk::Label::builder().xalign(0.0).build();
    name.add_css_class("dim-label");
    let flow = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .max_children_per_line(10)
        .min_children_per_line(5)
        .column_spacing(12)
        .row_spacing(12)
        .homogeneous(false)
        .halign(gtk::Align::Start)
        .build();
    let mut leader: Option<gtk::ToggleButton> = None;
    for t in theme::themes_for(mode) {
        let button = gtk::ToggleButton::builder()
            .tooltip_text(&t.name)
            .active(t.id == selected)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .width_request(40)
            .height_request(40)
            .build();
        button.add_css_class("theme-swatch");
        button.update_property(&[
            gtk::accessible::Property::Label(&t.name),
            gtk::accessible::Property::Description(&t.description),
        ]);
        // The swatch's own colour comes from the theme data, not a literal.
        if let Some(color) = &t.swatch {
            let css = gtk::CssProvider::new();
            css.load_from_string(&format!("button {{ background: {color}; }}"));
            #[allow(deprecated)]
            button
                .style_context()
                .add_provider(&css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 2);
        }
        if let Some(l) = &leader {
            button.set_group(Some(l));
        } else {
            leader = Some(button.clone());
        }
        if t.id == selected {
            name.set_label(&t.name);
        }
        let (id, label) = (t.id.clone(), t.name.clone());
        let name = name.clone();
        button.connect_toggled(move |btn| {
            if !btn.is_active() {
                return;
            }
            prefs::update(|p| match mode {
                "light" => p.light_theme = id.clone(),
                _ => p.dark_theme = id.clone(),
            });
            name.set_label(&label);
            theme::apply();
        });
        flow.append(&button);
    }
    b.append(&flow);
    b.append(&name);
    row.set_child(Some(&b));
    row.upcast()
}
