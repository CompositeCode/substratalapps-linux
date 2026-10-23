//! ACC-01 Account & profile: identity from `GET /users/me`; the editable
//! profile comes from the spec scaffold until it's built.

use std::rc::Rc;

use adw::prelude::*;
use substratal_core::screens::Screen;

use crate::shell::Shell;
use crate::widgets;

pub fn page(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    let outer = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    if let Some(me) = shell.ctx.me.borrow().as_ref() {
        let group = adw::PreferencesGroup::builder()
            .title("Signed in as")
            .build();
        let email = adw::ActionRow::builder()
            .title(gtk::glib::markup_escape_text(&me.email))
            .subtitle("Email")
            .build();
        email.add_suffix(&widgets::status_badge(me.status.key()));
        group.add(&email);
        let id = adw::ActionRow::builder().title("User ID").build();
        id.add_suffix(&widgets::id_chip(&me.id));
        group.add(&id);
        group.add(
            &adw::ActionRow::builder()
                .title("Two-factor authentication")
                .subtitle(if me.mfa_enabled { "On" } else { "Off" })
                .build(),
        );
        let column = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_top(24)
            .margin_start(16)
            .margin_end(16)
            .build();
        column.append(&group);
        outer.append(
            &adw::Clamp::builder()
                .maximum_size(760)
                .child(&column)
                .build(),
        );
    }
    outer.append(&super::scaffold_content(shell, screen));
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&outer)
        .build()
        .upcast()
}
