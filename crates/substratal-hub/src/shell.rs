//! The main window (linux/PLATFORM-SPEC.md §2): `AdwApplicationWindow` with
//! an `AdwNavigationSplitView` (sidebar · content), collapsing below 600 sp,
//! the GLB-03 system-state banners, and the window actions.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use substratal_core::nav::{self, Section};
use substratal_core::version::{self, VersionStatus};
use substratal_core::{config, screens};

use crate::context::AppContext;
use crate::{dialogs, pages, prefs, runtime};

/// A window action's handler.
type ShellAction = Box<dyn Fn(&Rc<Shell>)>;

pub struct Shell {
    pub window: adw::ApplicationWindow,
    pub ctx: Rc<AppContext>,
    split: adw::NavigationSplitView,
    sidebar: gtk::ListBox,
    content_page: adw::NavigationPage,
    title: adw::WindowTitle,
    body: adw::Bin,
    toasts: adw::ToastOverlay,
    update_banner: adw::Banner,
    offline_banner: adw::Banner,
    route: RefCell<String>,
    history: RefCell<Vec<String>>,
}

impl Shell {
    pub fn new(app: &adw::Application, ctx: Rc<AppContext>) -> Rc<Self> {
        let p = prefs::get();
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Substratal")
            .default_width(p.width)
            .default_height(p.height)
            .width_request(360)
            .height_request(294)
            .icon_name(crate::APP_ID)
            .build();

        // ── Sidebar ─────────────────────────────────────────────────────
        let sidebar = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();
        sidebar.add_css_class("navigation-sidebar");
        let sidebar_header = adw::HeaderBar::new();
        sidebar_header.set_title_widget(Some(&adw::WindowTitle::new("Substratal", "")));
        let search_button = gtk::Button::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search (Ctrl+K)")
            .action_name("win.command-palette")
            .build();
        sidebar_header.pack_start(&search_button);
        sidebar_header.pack_end(&primary_menu_button());
        let sidebar_view = adw::ToolbarView::new();
        sidebar_view.add_top_bar(&sidebar_header);
        sidebar_view.set_content(Some(
            &gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .child(&sidebar)
                .build(),
        ));
        let sidebar_page = adw::NavigationPage::builder()
            .title("Substratal")
            .tag("sidebar")
            .child(&sidebar_view)
            .build();

        // ── Content ─────────────────────────────────────────────────────
        let title = adw::WindowTitle::new("", "");
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&title));
        let help = gtk::Button::builder()
            .icon_name("help-browser-symbolic")
            .tooltip_text("Help for this screen (F1)")
            .action_name("win.help")
            .build();
        let refresh = gtk::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Refresh (Ctrl+R)")
            .action_name("win.refresh")
            .build();
        header.pack_end(&help);
        header.pack_end(&refresh);

        let update_banner = adw::Banner::builder()
            .button_label("Update")
            .revealed(false)
            .build();
        let offline_banner = adw::Banner::builder()
            .title("You're offline. Showing what was loaded; changes are paused.")
            .revealed(false)
            .build();
        let env_banner = adw::Banner::new("");
        if let Some(label) = ctx.api.environment().label() {
            env_banner.set_title(&format!("Connected to {label}, not production"));
            env_banner.set_revealed(true);
        }
        let preview_banner =
            adw::Banner::new("Preview mode: layout only, nothing is loaded from the API");
        preview_banner.set_revealed(ctx.is_preview());

        let body = adw::Bin::new();
        let content_view = adw::ToolbarView::new();
        content_view.add_top_bar(&header);
        for b in [
            &preview_banner,
            &env_banner,
            &update_banner,
            &offline_banner,
        ] {
            content_view.add_top_bar(b);
        }
        content_view.set_content(Some(&body));
        let content_page = adw::NavigationPage::builder()
            .title("")
            .tag("content")
            .child(&content_view)
            .build();

        let split = adw::NavigationSplitView::builder()
            .sidebar(&sidebar_page)
            .content(&content_page)
            .min_sidebar_width(220.0)
            .build();
        let toasts = adw::ToastOverlay::new();
        toasts.set_child(Some(&split));
        window.set_content(Some(&toasts));

        // ≤ 600 sp: the sidebar becomes the first page (PLATFORM-SPEC §2).
        let narrow =
            adw::Breakpoint::new(adw::BreakpointCondition::parse("max-width: 600sp").unwrap());
        narrow.add_setter(&split, "collapsed", Some(&true.to_value()));
        window.add_breakpoint(narrow);

        let shell = Rc::new(Shell {
            window,
            ctx,
            split,
            sidebar,
            content_page,
            title,
            body,
            toasts,
            update_banner,
            offline_banner,
            route: RefCell::new(String::new()),
            history: RefCell::new(Vec::new()),
        });
        shell.build_sidebar();
        shell.install_actions();
        shell.watch_version();
        shell.watch_network();
        shell.window.connect_close_request(|w| {
            let (width, height) = (w.default_width(), w.default_height());
            prefs::update(|p| {
                p.width = width;
                p.height = height;
            });
            glib::Propagation::Proceed
        });
        shell
    }

    fn build_sidebar(self: &Rc<Self>) {
        while let Some(row) = self.sidebar.row_at_index(0) {
            self.sidebar.remove(&row);
        }
        let caps = self.ctx.caps.borrow().clone();
        let mut first_of_section = Vec::new();
        for (section, items) in nav::visible_sections(&caps) {
            for (i, item) in items.iter().enumerate() {
                let row_box = gtk::Box::builder()
                    .spacing(12)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .build();
                row_box.append(&gtk::Image::from_icon_name(item.icon));
                let label = gtk::Label::builder()
                    .label(item.title)
                    .xalign(0.0)
                    .hexpand(true)
                    .build();
                row_box.append(&label);
                let row = gtk::ListBoxRow::builder()
                    .child(&row_box)
                    .name(item.screen_id)
                    .build();
                row.update_property(&[gtk::accessible::Property::Label(item.title)]);
                if i == 0 && section != Section::Personal {
                    first_of_section.push((row.clone(), section));
                }
                self.sidebar.append(&row);
            }
        }
        // Section headings above Administration and Developer.
        self.sidebar.set_header_func(move |row, _| {
            let heading = first_of_section
                .iter()
                .find(|(r, _)| r == row)
                .map(|(_, s)| s.title());
            row.set_header(
                heading
                    .map(|h| {
                        let l = gtk::Label::builder()
                            .label(h)
                            .xalign(0.0)
                            .margin_start(12)
                            .margin_top(12)
                            .margin_bottom(6)
                            .build();
                        l.add_css_class("heading");
                        l.add_css_class("dim-label");
                        l.upcast::<gtk::Widget>()
                    })
                    .as_ref(),
            );
        });
        let weak = Rc::downgrade(self);
        self.sidebar.connect_row_activated(move |_, row| {
            if let Some(shell) = weak.upgrade()
                && let Some(item) = nav::TREE
                    .iter()
                    .find(|i| i.screen_id == row.widget_name().as_str())
            {
                shell.navigate(item.route);
                shell.split.set_show_content(true);
            }
        });
    }

    pub fn current_route(&self) -> String {
        self.route.borrow().clone()
    }

    pub fn current_screen(&self) -> Option<&'static screens::Screen> {
        screens::match_route(&self.route.borrow(), false).and_then(|m| screens::get(&m.screen_id))
    }

    /// Shows the screen for `route`. Unknown routes show "Not found"; a
    /// route the person can't use shows Forbidden, not a redirect
    /// (INFORMATION-ARCHITECTURE §4).
    pub fn navigate(self: &Rc<Self>, route: &str) {
        let previous = self.route.replace(route.to_string());
        if !previous.is_empty() && previous != route {
            self.history.borrow_mut().push(previous);
        }
        let matched = screens::match_route(route, false);
        let (widget, title, subtitle) = match matched
            .as_ref()
            .and_then(|m| screens::get(&m.screen_id).map(|s| (s, m)))
        {
            Some((screen, m)) => {
                let allowed = nav::item_for_route(route)
                    .is_none_or(|item| item.is_visible(&self.ctx.caps.borrow()));
                let w = if allowed {
                    pages::build(self, screen, &m.params)
                } else {
                    widgets::status_page(
                        "action-unavailable-symbolic",
                        "You don't have access to this",
                        "Ask an administrator if you need it.",
                    )
                };
                (w, screen.title.clone(), screen.id.clone())
            }
            None => (
                widgets::status_page("find-location-symbolic", "Page not found", route),
                "Not found".into(),
                String::new(),
            ),
        };
        self.title.set_title(&title);
        self.title.set_subtitle(&subtitle);
        self.content_page.set_title(&title);
        self.window
            .set_title(Some(&format!("{title} — Substratal")));
        self.body.set_child(Some(&widget));
        self.select_sidebar_row(route);
    }

    fn select_sidebar_row(&self, route: &str) {
        let target = nav::item_for_route(route).map(|i| i.screen_id);
        let mut i = 0;
        while let Some(row) = self.sidebar.row_at_index(i) {
            if Some(row.widget_name().as_str()) == target {
                self.sidebar.select_row(Some(&row));
                return;
            }
            i += 1;
        }
        self.sidebar.select_row(None::<&gtk::ListBoxRow>);
    }

    pub fn toast(&self, text: &str) {
        self.toasts.add_toast(adw::Toast::new(text));
    }

    /// Opens a URL with the OpenURI portal.
    pub fn open_uri(&self, uri: &str) {
        gtk::UriLauncher::new(uri).launch(Some(&self.window), gio::Cancellable::NONE, |r| {
            if let Err(e) = r {
                eprintln!("substratal: couldn't open link: {e}");
            }
        });
    }

    fn help_for_current(&self) {
        let url = self
            .current_screen()
            .map(|s| s.help_url())
            .unwrap_or_else(|| config::HELP_BASE.to_string());
        self.open_uri(&url);
    }

    fn install_actions(self: &Rc<Self>) {
        let win = &self.window;
        let add = |name: &str, f: ShellAction| {
            let action = gio::SimpleAction::new(name, None);
            let weak = Rc::downgrade(self);
            action.connect_activate(move |_, _| {
                if let Some(s) = weak.upgrade() {
                    f(&s);
                }
            });
            win.add_action(&action);
        };
        add("help", Box::new(|s| s.help_for_current()));
        add("command-palette", Box::new(dialogs::command_palette));
        add("search", Box::new(dialogs::command_palette));
        add("show-shortcuts", Box::new(dialogs::shortcuts));
        add("about", Box::new(dialogs::about));
        add("preferences", Box::new(|s| s.navigate("/settings")));
        add("refresh", Box::new(|s| s.navigate(&s.current_route())));
        add(
            "toggle-sidebar",
            Box::new(|s| s.split.set_collapsed(!s.split.is_collapsed())),
        );
        add(
            "back",
            Box::new(|s| {
                let prev = s.history.borrow_mut().pop();
                if let Some(prev) = prev {
                    s.navigate(&prev);
                    s.history.borrow_mut().pop();
                }
            }),
        );
        add("sign-out", Box::new(|s| s.sign_out()));
        for name in ["new", "undo", "delete", "mark-all-read"] {
            add(
                name,
                Box::new(|s| s.toast("Nothing to do on this screen yet")),
            );
        }

        // win.go::SCREEN-ID (Alt+1…6) and win.navigate::/route.
        for (name, by_id) in [("go", true), ("navigate", false)] {
            let action = gio::SimpleAction::new(name, Some(glib::VariantTy::STRING));
            let weak = Rc::downgrade(self);
            action.connect_activate(move |_, param| {
                let (Some(s), Some(arg)) = (weak.upgrade(), param.and_then(|p| p.get::<String>()))
                else {
                    return;
                };
                let route = if by_id {
                    screens::get(&arg).map(|sc| sc.route.clone())
                } else {
                    Some(arg)
                };
                if let Some(route) = route {
                    s.navigate(&route);
                }
            });
            win.add_action(&action);
        }
    }

    fn sign_out(self: &Rc<Self>) {
        let ctx = self.ctx.clone();
        let app = self.window.application().and_downcast::<adw::Application>();
        let api = ctx.api.clone();
        runtime::spawn(async move { api.logout().await }, move |r| {
            if let Err(e) = r {
                eprintln!("substratal: sign-out: {e}");
            }
            ctx.me.replace(None);
            ctx.caps.replace(Default::default());
            if let Some(app) = app {
                crate::show_sign_in(&app, ctx, None);
            }
        });
    }

    /// GLB-03 Update available (banner) and Update required (blocking).
    fn watch_version(self: &Rc<Self>) {
        let mut rx = self.ctx.api.version_status();
        let (tx, ui_rx) = async_channel::unbounded::<VersionStatus>();
        runtime::spawn(
            async move {
                while rx.changed().await.is_ok() {
                    let v = rx.borrow().clone();
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
            },
            |_| {},
        );
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            while let Ok(status) = ui_rx.recv().await {
                let Some(s) = weak.upgrade() else { break };
                match status {
                    VersionStatus::Current => s.update_banner.set_revealed(false),
                    VersionStatus::Deprecated { sunset, .. } => {
                        let when = sunset
                            .map(version::fmt_date)
                            .unwrap_or_else(|| "soon".into());
                        s.update_banner.set_title(&format!("A newer version of Substratal is available. This version stops working {when}."));
                        s.update_banner.set_revealed(true);
                    }
                    VersionStatus::Unsupported { .. } => {
                        s.body.set_child(Some(&widgets::status_page(
                            "software-update-urgent-symbolic",
                            "Update required",
                            "This version of Substratal no longer works with the service. Update the app to keep using it.",
                        )));
                        s.title.set_title("Update required");
                    }
                }
            }
        });
        let weak = Rc::downgrade(self);
        self.update_banner.connect_button_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.open_uri("appstream://com.substratalapps.Hub");
            }
        });
    }

    /// GLB-03 Offline banner from GNetworkMonitor.
    fn watch_network(self: &Rc<Self>) {
        let monitor = gio::NetworkMonitor::default();
        self.offline_banner
            .set_revealed(!monitor.is_network_available());
        let weak = Rc::downgrade(self);
        monitor.connect_network_changed(move |_, available| {
            if let Some(s) = weak.upgrade() {
                s.offline_banner.set_revealed(!available);
            }
        });
    }
}

fn primary_menu_button() -> gtk::MenuButton {
    let menu = gio::Menu::new();
    let section = gio::Menu::new();
    section.append(Some("_Preferences"), Some("win.preferences"));
    section.append(Some("_Keyboard Shortcuts"), Some("win.show-shortcuts"));
    section.append(Some("_Help"), Some("win.help"));
    section.append(Some("_About Substratal"), Some("win.about"));
    menu.append_section(None, &section);
    let account = gio::Menu::new();
    account.append(Some("_Sign Out"), Some("win.sign-out"));
    menu.append_section(None, &account);
    gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text("Main menu")
        .primary(true)
        .build()
}

use crate::widgets;
