//! Spec-driven scaffold for screens not hand-built yet.
//!
//! Every screen in `linux/SCREENS.md` has a route, an archetype, the
//! elements with their GTK/libadwaita control, the API calls and the states.
//! This page lays a screen out by its archetype and renders each element
//! with the control the spec names, so every one of the 75 screens is
//! reachable, navigable and reviewable now, and each hand-built screen
//! replaces its scaffold without changing routes or navigation.

use std::rc::Rc;

use adw::prelude::*;
use substratal_core::screens::{self, Element, Screen};

use crate::shell::Shell;
use crate::widgets;

pub fn page(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&content(shell, screen))
        .build()
        .upcast()
}

/// The scaffold without its scroller, for hand-built screens that embed it.
pub fn content(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    let column = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(16)
        .margin_end(16)
        .build();
    column.append(&header(shell, screen));

    match screen.archetype.as_str() {
        "table" | "list" | "grid" => column.append(&collection(screen)),
        "detail" => column.append(&detail(shell, screen)),
        "settings" => column.append(&form_preview(screen)),
        "form" | "wizard" | "confirm" | "secret" | "popover" | "handoff" => {
            column.append(&dialog_launcher(shell, screen))
        }
        _ => {}
    }
    column.append(&spec_groups(screen));

    adw::Clamp::builder()
        .maximum_size(
            if matches!(screen.archetype.as_str(), "table" | "list" | "grid") {
                1200
            } else {
                760
            },
        )
        .child(&column)
        .build()
        .upcast()
}

fn header(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .build();
    let title = gtk::Label::builder()
        .label(&screen.title)
        .xalign(0.0)
        .wrap(true)
        .build();
    title.add_css_class("title-1");
    b.append(&title);
    if !screen.purpose.is_empty() {
        let p = gtk::Label::builder()
            .label(&screen.purpose)
            .xalign(0.0)
            .wrap(true)
            .build();
        p.add_css_class("dim-label");
        b.append(&p);
    }
    let tags = gtk::Box::builder().spacing(8).build();
    tags.append(&widgets::id_chip(&screen.id));
    for t in [&screen.archetype, &screen.phase] {
        let l = gtk::Label::new(Some(t));
        l.add_css_class("caption");
        l.add_css_class("dim-label");
        tags.append(&l);
    }
    let spacer = gtk::Box::builder().hexpand(true).build();
    tags.append(&spacer);
    let help = gtk::Button::builder().label("Help").build();
    help.add_css_class("flat");
    let s = shell.clone();
    let url = screen.help_url();
    help.connect_clicked(move |_| s.open_uri(&url));
    tags.append(&help);
    b.append(&tags);
    if !screen.who.is_empty() {
        let who = gtk::Label::builder()
            .label(format!("Who: {}", screen.who))
            .xalign(0.0)
            .wrap(true)
            .build();
        who.add_css_class("caption");
        who.add_css_class("spec-caption");
        b.append(&who);
    }
    b.upcast()
}

fn collection(screen: &Screen) -> gtk::Widget {
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    if screen.elements.iter().any(|e| e.kind == "search") || screen.archetype != "grid" {
        let search = gtk::SearchEntry::builder()
            .placeholder_text(format!("Search {}", screen.title.to_lowercase()))
            .build();
        b.append(&search);
    }
    let chips: Vec<&Element> = screen
        .elements
        .iter()
        .filter(|e| e.kind == "chips" || e.kind.starts_with("segmented"))
        .collect();
    for c in chips {
        let opts = options(&c.data);
        if opts.len() < 2 {
            continue;
        }
        let row = gtk::Box::builder().spacing(6).build();
        for (i, opt) in opts.into_iter().take(6).enumerate() {
            let t = gtk::ToggleButton::builder()
                .label(opt)
                .active(i == 0)
                .build();
            t.add_css_class("pill");
            row.append(&t);
        }
        b.append(&row);
    }
    let empty = adw::StatusPage::builder()
        .icon_name("folder-documents-symbolic")
        .title("Nothing loaded yet")
        .description(widgets::glib_escape(&if screen.loads.is_empty() {
            "This list loads from the API.".to_string()
        } else {
            format!("Loads {}", screen.loads.replace('`', ""))
        }))
        .build();
    empty.add_css_class("compact");
    let frame = gtk::Frame::builder().child(&empty).build();
    frame.add_css_class("card");
    b.append(&frame);
    b.upcast()
}

/// A detail screen: a view switcher over the tab screens nested under its route.
fn detail(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    let prefix = screen.route.clone();
    let tabs: Vec<&'static Screen> = screens::all()
        .iter()
        .filter(|s| {
            s.archetype == "tab"
                && s.on_linux()
                && s.route.starts_with(&prefix)
                && s.route != prefix
        })
        .collect();
    if tabs.is_empty() {
        return form_preview(screen);
    }
    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    stack.add_titled(&form_preview(screen), Some(&screen.id), "Overview");
    for t in &tabs {
        let short = t.title.rsplit(" — ").next().unwrap_or(&t.title);
        stack.add_titled(&tab_body(shell, t), Some(&t.id), short);
    }
    let switcher = gtk::StackSwitcher::builder()
        .stack(&stack)
        .halign(gtk::Align::Center)
        .build();
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    b.append(&switcher);
    b.append(&stack);
    b.upcast()
}

fn tab_body(shell: &Rc<Shell>, tab: &'static Screen) -> gtk::Widget {
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    let row = gtk::Box::builder().spacing(8).build();
    row.append(&widgets::id_chip(&tab.id));
    let p = gtk::Label::builder()
        .label(&tab.purpose)
        .xalign(0.0)
        .wrap(true)
        .hexpand(true)
        .build();
    p.add_css_class("dim-label");
    row.append(&p);
    let help = gtk::Button::builder()
        .icon_name("help-browser-symbolic")
        .tooltip_text("Help for this tab")
        .build();
    help.add_css_class("flat");
    let s = shell.clone();
    let url = tab.help_url();
    help.connect_clicked(move |_| s.open_uri(&url));
    row.append(&help);
    b.append(&row);
    b.append(&form_preview(tab));
    b.upcast()
}

/// A screen that opens as a dialog: explain where it's reached from and offer a preview.
fn dialog_launcher(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    let group = adw::PreferencesGroup::builder()
        .title("Dialog")
        .description(widgets::glib_escape(&format!(
            "Shown as {} ({}).",
            if screen.container.is_empty() {
                "a dialog"
            } else {
                &screen.container
            },
            screen.archetype
        )))
        .build();
    let row = adw::ActionRow::builder()
        .title("Preview this dialog")
        .subtitle("Controls only; nothing is sent to the API")
        .activatable(true)
        .build();
    row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
    let s = shell.clone();
    row.connect_activated(move |_| {
        let d = adw::Dialog::builder()
            .title(&screen.title)
            .content_width(480)
            .build();
        let view = adw::ToolbarView::new();
        view.add_top_bar(&adw::HeaderBar::new());
        let clamp = adw::Clamp::builder()
            .maximum_size(440)
            .margin_top(12)
            .margin_bottom(24)
            .margin_start(12)
            .margin_end(12)
            .child(&form_preview(screen))
            .build();
        view.set_content(Some(
            &gtk::ScrolledWindow::builder()
                .propagate_natural_height(true)
                .child(&clamp)
                .build(),
        ));
        d.set_child(Some(&view));
        d.present(Some(&s.window));
    });
    group.add(&row);
    group.upcast()
}

/// Renders the spec's elements with the control each one names.
fn form_preview(screen: &Screen) -> gtk::Widget {
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .build();
    let group = adw::PreferencesGroup::new();
    let mut buttons = gtk::Box::builder()
        .spacing(12)
        .halign(gtk::Align::End)
        .build();
    let mut has_rows = false;
    for e in &screen.elements {
        let kind = e.kind.as_str();
        let title = gtk::glib::markup_escape_text(&e.name).to_string();
        let w: Option<gtk::Widget> = if kind.contains("password") {
            Some(
                adw::PasswordEntryRow::builder()
                    .title(&title)
                    .build()
                    .upcast(),
            )
        } else if kind.ends_with("-field") || kind == "text-area" || kind == "search" {
            Some(adw::EntryRow::builder().title(&title).build().upcast())
        } else if kind == "toggle" || kind == "checkbox" {
            Some(
                adw::SwitchRow::builder()
                    .title(&title)
                    .subtitle(gtk::glib::markup_escape_text(&e.data))
                    .build()
                    .upcast(),
            )
        } else if kind == "select" || kind == "radio-group" || kind.starts_with("segmented") {
            let opts = options(&e.data);
            let model = gtk::StringList::new(&opts.iter().map(String::as_str).collect::<Vec<_>>());
            Some(
                adw::ComboRow::builder()
                    .title(&title)
                    .model(&model)
                    .build()
                    .upcast(),
            )
        } else if kind == "primary-button" || kind == "button" || kind == "destructive-button" {
            let btn = gtk::Button::builder().label(&e.name).build();
            match kind {
                "primary-button" => btn.add_css_class("suggested-action"),
                "destructive-button" => btn.add_css_class("destructive-action"),
                _ => {}
            }
            btn.add_css_class("pill");
            btn.set_sensitive(false);
            btn.set_tooltip_text(Some(&format!("{} — not wired yet", e.data)));
            buttons.append(&btn);
            None
        } else if kind == "banner" || kind == "caption" || kind == "text" {
            let l = gtk::Label::builder()
                .label(format!("{} — {}", e.name, e.data))
                .xalign(0.0)
                .wrap(true)
                .build();
            l.add_css_class("dim-label");
            b.append(&l);
            None
        } else {
            let row = adw::ActionRow::builder()
                .title(&title)
                .subtitle(gtk::glib::markup_escape_text(&e.data))
                .build();
            row.add_suffix(&kind_label(kind));
            Some(row.upcast())
        };
        if let Some(w) = w {
            w.set_sensitive(!matches!(kind, "key-value" | "list" | "table"));
            group.add(&w);
            has_rows = true;
        }
    }
    if has_rows {
        b.append(&group);
    }
    if buttons.first_child().is_some() {
        b.append(&std::mem::replace(
            &mut buttons,
            gtk::Box::new(gtk::Orientation::Horizontal, 0),
        ));
    }
    b.upcast()
}

fn kind_label(kind: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(kind));
    l.add_css_class("caption");
    l.add_css_class("dim-label");
    l
}

/// Options named in a spec data cell: `status active | invited | suspended` → [active, invited, suspended].
fn options(data: &str) -> Vec<String> {
    let parts: Vec<String> = data
        .split(['|', '·', ','])
        .map(|p| p.trim().trim_matches('`').to_string())
        .filter(|p| !p.is_empty())
        .collect();
    let mut parts = parts;
    if let Some(first) = parts.first_mut()
        && let Some((_, last_word)) = first.rsplit_once(' ')
    {
        *first = last_word.to_string();
    }
    if parts.is_empty() {
        vec!["—".into()]
    } else {
        parts
    }
}

/// The screen's spec, for reviewers: API calls and states.
fn spec_groups(screen: &Screen) -> gtk::Widget {
    let expander = gtk::Expander::builder().label("Specification").build();
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .margin_top(12)
        .build();
    if !screen.container.is_empty() || !screen.loads.is_empty() {
        let g = adw::PreferencesGroup::builder()
            .title("Layout and data")
            .build();
        if !screen.container.is_empty() {
            g.add(
                &adw::ActionRow::builder()
                    .title("Container")
                    .subtitle(gtk::glib::markup_escape_text(&screen.container))
                    .build(),
            );
        }
        if !screen.loads.is_empty() {
            g.add(
                &adw::ActionRow::builder()
                    .title("Loads")
                    .subtitle(gtk::glib::markup_escape_text(
                        &screen.loads.replace('`', ""),
                    ))
                    .subtitle_selectable(true)
                    .build(),
            );
        }
        b.append(&g);
    }
    if !screen.elements.is_empty() {
        let g = adw::PreferencesGroup::builder().title("Elements").build();
        for e in &screen.elements {
            let subtitle = [e.control.as_str(), e.data.as_str(), e.rules.as_str()]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" · ");
            let row = adw::ActionRow::builder()
                .title(gtk::glib::markup_escape_text(&e.name))
                .subtitle(gtk::glib::markup_escape_text(&subtitle))
                .subtitle_lines(3)
                .build();
            row.add_suffix(&kind_label(&e.kind));
            g.add(&row);
        }
        b.append(&g);
    }
    if !screen.actions.is_empty() {
        let g = adw::PreferencesGroup::builder().title("Actions").build();
        for a in &screen.actions {
            let subtitle = [a.call.as_str(), a.result.as_str()]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" → ");
            g.add(
                &adw::ActionRow::builder()
                    .title(gtk::glib::markup_escape_text(&a.action))
                    .subtitle(gtk::glib::markup_escape_text(&subtitle))
                    .subtitle_lines(4)
                    .build(),
            );
        }
        b.append(&g);
    }
    if !screen.states.is_empty() {
        let l = gtk::Label::builder()
            .label(format!("States: {}", screen.states))
            .xalign(0.0)
            .wrap(true)
            .build();
        l.add_css_class("caption");
        l.add_css_class("dim-label");
        b.append(&l);
    }
    expander.set_child(Some(&b));
    expander.upcast()
}
