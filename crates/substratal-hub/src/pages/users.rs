//! USR-01 Users: `GtkColumnView` (.data-table) over `GET /users`.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use substratal_core::models::User;
use substratal_core::screens::Screen;

use crate::shell::Shell;
use crate::{runtime, widgets};

const PAGE_SIZE: u32 = 50;

pub fn page(shell: &Rc<Shell>, screen: &'static Screen) -> gtk::Widget {
    if shell.ctx.is_preview() {
        return super::scaffold_page(shell, screen);
    }
    let store = gio::ListStore::new::<glib::BoxedAnyObject>();
    let selection = gtk::MultiSelection::new(Some(store.clone()));
    let view = gtk::ColumnView::builder()
        .model(&selection)
        .show_row_separators(true)
        .reorderable(true)
        .vexpand(true)
        .build();
    view.add_css_class("data-table");

    add_text_column(&view, "Email", true, |u| u.email.clone());
    add_widget_column(&view, "Status", |u| widgets::status_badge(u.status.key()));
    add_text_column(&view, "ID", false, |u| u.id.clone());
    add_text_column(&view, "Last sign-in", false, |u| {
        u.last_login_at.clone().unwrap_or_else(|| "—".into())
    });
    add_text_column(&view, "Created", false, |u| {
        u.created_at.clone().unwrap_or_default()
    });

    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search by name, email or usr_ ID")
        .hexpand(true)
        .build();
    let invite = gtk::Button::builder()
        .label("Invite user")
        .action_name("win.navigate")
        .action_target(&"/admin/users/invite".to_variant())
        .build();
    invite.add_css_class("suggested-action");
    invite.add_css_class("pill");
    invite.set_visible(shell.ctx.caps.borrow().can_manage_users());
    let more = gtk::Button::builder()
        .label("Load more")
        .halign(gtk::Align::Center)
        .visible(false)
        .build();
    let status = gtk::Label::builder().xalign(0.0).build();
    status.add_css_class("dim-label");

    let toolbar = gtk::Box::builder().spacing(12).build();
    toolbar.append(&search);
    toolbar.append(&invite);

    let cursor: Rc<RefCell<Option<String>>> = Rc::default();
    let load: Rc<dyn Fn(bool)> = {
        let (api, store, more, status, search, cursor) = (
            shell.ctx.api.clone(),
            store.clone(),
            more.clone(),
            status.clone(),
            search.clone(),
            cursor.clone(),
        );
        Rc::new(move |append: bool| {
            let q = search.text().to_string();
            let c = if append {
                cursor.borrow().clone()
            } else {
                None
            };
            status.set_label("Loading…");
            let api = api.clone();
            let (store, more, status, cursor) =
                (store.clone(), more.clone(), status.clone(), cursor.clone());
            runtime::spawn(
                async move { api.list_users(Some(&q), c.as_deref(), PAGE_SIZE).await },
                move |r| match r {
                    Ok(page) => {
                        if !append {
                            store.remove_all();
                        }
                        for u in page.data {
                            store.append(&glib::BoxedAnyObject::new(u));
                        }
                        *cursor.borrow_mut() = page.page.next_cursor;
                        more.set_visible(page.page.has_more);
                        status.set_label(&format!("{} loaded", store.n_items()));
                    }
                    Err(e) => {
                        let msg = match &e {
                            substratal_core::Error::Api(a) => format!(
                                "{} (request {})",
                                a.user_message(),
                                a.request_id.as_deref().unwrap_or("—")
                            ),
                            other => other.to_string(),
                        };
                        status.set_label(&msg);
                    }
                },
            );
        })
    };
    let l = load.clone();
    search.connect_search_changed(move |_| l(false));
    let l = load.clone();
    more.connect_clicked(move |_| l(true));
    let s = shell.clone();
    view.connect_activate(move |v, pos| {
        let Some(obj) = v
            .model()
            .and_then(|m| m.item(pos))
            .and_downcast::<glib::BoxedAnyObject>()
        else {
            return;
        };
        let id = obj.borrow::<User>().id.clone();
        s.navigate(&format!("/admin/users/{id}"));
    });
    load(false);

    let frame = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&view)
        .build();
    frame.add_css_class("card");
    let column = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(16)
        .margin_end(16)
        .build();
    column.append(&toolbar);
    column.append(&frame);
    column.append(&status);
    column.append(&more);
    column.upcast()
}

fn add_text_column(
    view: &gtk::ColumnView,
    title: &str,
    expand: bool,
    text: impl Fn(&User) -> String + 'static,
) {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let label = gtk::Label::builder()
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        item.downcast_ref::<gtk::ListItem>()
            .unwrap()
            .set_child(Some(&label));
    });
    factory.connect_bind(move |_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let (Some(obj), Some(label)) = (
            item.item().and_downcast::<glib::BoxedAnyObject>(),
            item.child().and_downcast::<gtk::Label>(),
        ) else {
            return;
        };
        label.set_label(&text(&obj.borrow::<User>()));
    });
    view.append_column(
        &gtk::ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(expand)
            .resizable(true)
            .build(),
    );
}

fn add_widget_column(
    view: &gtk::ColumnView,
    title: &str,
    make: impl Fn(&User) -> gtk::Widget + 'static,
) {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_bind(move |_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(obj) = item.item().and_downcast::<glib::BoxedAnyObject>() {
            item.set_child(Some(&make(&obj.borrow::<User>())));
        }
    });
    view.append_column(
        &gtk::ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .build(),
    );
}
