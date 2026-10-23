//! GLB-01 Command palette, the Keyboard Shortcuts dialog and About.

use std::rc::Rc;

use adw::prelude::*;
use substratal_core::{config, nav, screens, shortcuts};

use crate::shell::Shell;

/// GLB-01: jump to any screen the person can open, by name, ID or route.
pub fn command_palette(shell: &Rc<Shell>) {
    let dialog = adw::Dialog::builder()
        .title("Go to")
        .content_width(520)
        .content_height(480)
        .build();
    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search screens by name, ID or route")
        .hexpand(true)
        .build();
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Browse)
        .build();
    list.add_css_class("boxed-list");

    let caps = shell.ctx.caps.borrow().clone();
    let targets: Vec<&'static screens::Screen> = screens::all()
        .iter()
        .filter(|s| {
            s.on_linux() && !s.is_auth() && !s.route.contains(':') && s.route.starts_with('/')
        })
        .filter(|s| nav::item_for_route(&s.route).is_none_or(|i| i.is_visible(&caps)))
        .collect();
    for s in &targets {
        let row = adw::ActionRow::builder()
            .title(gtk::glib::markup_escape_text(&s.title))
            .subtitle(gtk::glib::markup_escape_text(&s.route))
            .activatable(true)
            .build();
        row.add_suffix(&crate::widgets::id_chip(&s.id));
        row.set_widget_name(&s.route);
        list.append(&row);
    }
    let needle = Rc::new(std::cell::RefCell::new(String::new()));
    let n = needle.clone();
    list.set_filter_func(move |row| {
        let q = n.borrow();
        if q.is_empty() {
            return true;
        }
        let Some(r) = row.downcast_ref::<adw::ActionRow>() else {
            return true;
        };
        let hay = format!(
            "{} {} {}",
            r.title(),
            r.subtitle().unwrap_or_default(),
            r.widget_name()
        )
        .to_lowercase();
        q.split_whitespace().all(|w| hay.contains(w))
    });
    let l = list.clone();
    search.connect_search_changed(move |e| {
        *needle.borrow_mut() = e.text().to_lowercase();
        l.invalidate_filter();
    });
    let go = {
        let shell = shell.clone();
        let dialog = dialog.clone();
        move |route: String| {
            dialog.close();
            shell.navigate(&route);
        }
    };
    let go1 = go.clone();
    list.connect_row_activated(move |_, row| go1(row.widget_name().to_string()));
    let l = list.clone();
    search.connect_activate(move |_| {
        let mut i = 0;
        while let Some(row) = l.row_at_index(i) {
            if row.is_child_visible() {
                go(row.widget_name().to_string());
                return;
            }
            i += 1;
        }
    });

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&search);
    content.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&list)
            .build(),
    );
    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(&content));
    dialog.set_child(Some(&view));
    dialog.present(Some(&shell.window));
    search.grab_focus();
}

/// Keyboard shortcuts, from the same table that binds them.
pub fn shortcuts(shell: &Rc<Shell>) {
    let dialog = adw::Dialog::builder()
        .title("Keyboard Shortcuts")
        .content_width(480)
        .content_height(600)
        .build();
    let page = adw::PreferencesPage::new();
    let mut groups: Vec<(&str, adw::PreferencesGroup)> = Vec::new();
    for s in shortcuts::SHORTCUTS {
        let group = match groups.iter().find(|(g, _)| *g == s.group) {
            Some((_, g)) => g.clone(),
            None => {
                let g = adw::PreferencesGroup::builder().title(s.group).build();
                page.add(&g);
                groups.push((s.group, g.clone()));
                g
            }
        };
        let row = adw::ActionRow::builder().title(s.label).build();
        row.add_suffix(
            &gtk::ShortcutLabel::builder()
                .accelerator(s.accels.join(" "))
                .valign(gtk::Align::Center)
                .build(),
        );
        group.add(&row);
    }
    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(&page));
    dialog.set_child(Some(&view));
    dialog.present(Some(&shell.window));
}

pub fn about(shell: &Rc<Shell>) {
    let about = adw::AboutDialog::builder()
        .application_name("Substratal")
        .application_icon(crate::APP_ID)
        .developer_name("Composite Code")
        .version(crate::VERSION)
        .website(config::HELP_BASE)
        .support_url(config::HELP_BASE)
        .issue_url("https://github.com/CompositeCode/substratalapps-linux/issues")
        .license_type(gtk::License::MitX11)
        .comments(format!(
            "Substratal Hub for Linux.\nPlatform API version {} (Substratal-Version header) at {}.\nScreens from specs {}.",
            substratal_core::version::API_VERSION,
            shell.ctx.api.environment().base_url(),
            &screens::spec_commit()[..7.min(screens::spec_commit().len())],
        ))
        .build();
    about.add_link("Help center", config::HELP_BASE);
    about.add_link("Platform API reference", config::API_DOCS);
    about.present(Some(&shell.window));
}
