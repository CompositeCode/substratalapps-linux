//! Substratal Hub for Linux (GNOME).
//!
//! GTK 4 + libadwaita client of the Substratal Apps Platform API, built to
//! `linux/PLATFORM-SPEC.md` and `linux/SCREENS.md` in the specs package.
//! Every request sends `Substratal-Version: 1` to the host root
//! (`https://api.substratalapps.com`, no `/v1`); see `substratal_core::version`.

mod context;
mod dialogs;
mod pages;
mod prefs;
mod runtime;
mod shell;
mod sign_in;
mod theme;
mod widgets;

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use substratal_core::{nav, screens, shortcuts};

use crate::context::AppContext;
use crate::shell::Shell;

pub const APP_ID: &str = "com.substratalapps.Hub";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

thread_local! {
    static SHELL: RefCell<Option<Rc<Shell>>> = const { RefCell::new(None) };
    static PENDING_ROUTE: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn main() -> glib::ExitCode {
    gio::resources_register_include!("hub.gresource").expect("bundled resources");
    glib::set_application_name("Substratal");

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|app| {
        theme::init();
        for s in shortcuts::SHORTCUTS {
            if let Some(action) = s.action {
                app.set_accels_for_action(action, s.accels);
            }
        }
        let quit = gio::SimpleAction::new("quit", None);
        quit.connect_activate(glib::clone!(
            #[weak]
            app,
            move |_, _| app.quit()
        ));
        app.add_action(&quit);
    });

    app.connect_activate(activate);

    // Deep links: `substratal://admin/users` (x-scheme-handler/substratal,
    // DBusActivatable so they reach the running instance).
    app.connect_open(|app, files, _| {
        for f in files {
            let uri = f.uri();
            match screens::match_link(&uri) {
                Some(m) => {
                    let route = screens::get(&m.screen_id)
                        .map(|s| {
                            s.path_with(
                                &m.params
                                    .iter()
                                    .map(|(k, v)| (k.as_str(), v.as_str()))
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .unwrap_or_default();
                    open_route(route);
                }
                None => eprintln!("substratal: ignoring unknown link {uri}"),
            }
        }
        activate(app);
    });

    app.run()
}

fn open_route(route: String) {
    let shown = SHELL.with(|s| {
        s.borrow()
            .as_ref()
            .map(|shell| shell.navigate(&route))
            .is_some()
    });
    if !shown {
        PENDING_ROUTE.with(|p| *p.borrow_mut() = Some(route));
    }
}

fn activate(app: &adw::Application) {
    if let Some(w) = app.active_window() {
        w.present();
        return;
    }
    let ctx = AppContext::new();

    // Developer preview: `SUBSTRATAL_PREVIEW=superadmin` shows the shell as
    // that persona without signing in (screenshots, layout work). It only
    // changes what the UI shows; every API call still needs a real session.
    if let Some(persona) = ctx.preview.clone() {
        let caps = substratal_core::capabilities::personas::all()
            .into_iter()
            .find(|(name, _)| name.to_lowercase().replace(' ', "_") == persona)
            .map(|(_, c)| c)
            .unwrap_or_else(substratal_core::capabilities::personas::superadmin);
        *ctx.caps.borrow_mut() = caps;
        show_shell(app, ctx);
        return;
    }

    // Keep the application alive while the session is restored off the
    // main thread; otherwise it quits for want of a window.
    let hold = app.hold();
    let api = ctx.api.clone();
    runtime::spawn(
        async move {
            match api.restore().await {
                Ok(true) => Some(api.capabilities().await),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            }
        },
        glib::clone!(
            #[weak]
            app,
            move |restored| {
                let _hold = hold;
                match restored {
                    Some(Ok((me, caps))) => {
                        *ctx.me.borrow_mut() = Some(me);
                        *ctx.caps.borrow_mut() = caps;
                        show_shell(&app, ctx);
                    }
                    Some(Err(e)) => {
                        eprintln!("substratal: couldn't restore the session: {e}");
                        sign_in::present(&app, ctx, Some(e.to_string()));
                    }
                    None => sign_in::present(&app, ctx, None),
                }
            }
        ),
    );
}

/// Opens the main window for a signed-in (or preview) person.
pub fn show_shell(app: &adw::Application, ctx: Rc<AppContext>) {
    let shell = Shell::new(app, ctx);
    let start = PENDING_ROUTE
        .with(|p| p.borrow_mut().take())
        .or_else(|| std::env::var("SUBSTRATAL_ROUTE").ok());
    shell.navigate(start.as_deref().unwrap_or(nav::TREE[0].route));
    shell.window.present();
    SHELL.with(|s| *s.borrow_mut() = Some(shell));
}

/// Back to AUTH-01 after sign-out or a revoked session, keeping the destination.
pub fn show_sign_in(app: &adw::Application, ctx: Rc<AppContext>, message: Option<String>) {
    if let Some(shell) = SHELL.with(|s| s.borrow_mut().take()) {
        PENDING_ROUTE.with(|p| *p.borrow_mut() = Some(shell.current_route()));
        shell.window.close();
    }
    sign_in::present(app, ctx, message);
}
