//! AUTH-01 Sign in and AUTH-02 Two-factor challenge, from `ui/sign-in.blp`.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use substratal_core::models::LoginOutcome;
use substratal_core::{Error, config};

use crate::context::AppContext;
use crate::runtime;

fn obj<T: IsA<glib::Object>>(b: &gtk::Builder, id: &str) -> T {
    b.object(id)
        .unwrap_or_else(|| panic!("sign-in.ui is missing {id}"))
}

fn error_text(e: &Error) -> String {
    match e {
        Error::Api(a) if a.status == 429 => match a.retry_after {
            Some(d) => format!(
                "Too many attempts. Try again in {} seconds.",
                d.as_secs().max(1)
            ),
            None => a.user_message(),
        },
        Error::Api(a) => a.user_message(),
        Error::Network(_) => {
            "Couldn't reach Substratal. Check your connection and try again.".into()
        }
        Error::UpdateRequired { .. } => {
            "This version of Substratal no longer works with the service. Update the app.".into()
        }
        other => other.to_string(),
    }
}

pub fn present(app: &adw::Application, ctx: Rc<AppContext>, message: Option<String>) {
    let b = gtk::Builder::from_resource("/com/substratalapps/Hub/ui/sign-in.ui");
    let window: adw::Window = obj(&b, "sign_in_window");
    window.set_application(Some(app));
    let nav: adw::NavigationView = obj(&b, "nav");
    let mfa_page: adw::NavigationPage = obj(&b, "mfa_page");
    let email: adw::EntryRow = obj(&b, "email_row");
    let password: adw::PasswordEntryRow = obj(&b, "password_row");
    let error: gtk::Label = obj(&b, "error_label");
    let sign_in: gtk::Button = obj(&b, "sign_in_button");
    let code: adw::EntryRow = obj(&b, "code_row");
    let mfa_error: gtk::Label = obj(&b, "mfa_error_label");
    let verify: gtk::Button = obj(&b, "verify_button");
    let logo: gtk::Picture = obj(&b, "logo");
    let env_banner: adw::Banner = obj(&b, "env_banner");
    let forgot: gtk::LinkButton = obj(&b, "forgot_link");
    let sign_up: gtk::LinkButton = obj(&b, "sign_up_link");

    // BrandLockup above the form, swapped for the on-dark variant in dark mode.
    let set_logo = glib::clone!(
        #[weak]
        logo,
        move |sm: &adw::StyleManager| {
            let file = if sm.is_dark() {
                "lockup-horizontal-on-dark.svg"
            } else {
                "lockup-horizontal.svg"
            };
            logo.set_resource(Some(&format!("/com/substratalapps/Hub/brand/{file}")));
        }
    );
    let sm = adw::StyleManager::default();
    set_logo(&sm);
    sm.connect_dark_notify(set_logo);

    if let Some(label) = ctx.api.environment().label() {
        env_banner.set_title(&format!("Signing in to {label}, not production"));
        env_banner.set_revealed(true);
    }
    // AUTH-03 and AUTH-05 aren't built yet; until then they open on the web dashboard.
    forgot.set_uri(&format!("{}/forgot-password", config::DASHBOARD));
    sign_up.set_uri(&format!("{}/sign-up", config::DASHBOARD));

    if let Some(m) = message {
        error.set_label(&m);
        error.set_visible(true);
    }

    let help = gio::SimpleAction::new("help", None);
    help.connect_activate(glib::clone!(
        #[weak]
        window,
        #[weak]
        nav,
        move |_, _| {
            let id = nav
                .visible_page()
                .and_then(|p| p.tag())
                .map(|t| t.to_string())
                .unwrap_or_else(|| "AUTH-01".into());
            gtk::UriLauncher::new(&config::help_url(&id)).launch(
                Some(&window),
                gio::Cancellable::NONE,
                |_| {},
            );
        }
    ));
    let group = gio::SimpleActionGroup::new();
    group.add_action(&help);
    window.insert_action_group("signin", Some(&group));
    let f1 = gtk::ShortcutController::new();
    f1.add_shortcut(gtk::Shortcut::new(
        gtk::ShortcutTrigger::parse_string("F1"),
        Some(gtk::NamedAction::new("signin.help")),
    ));
    window.add_controller(f1);

    // Sign in is disabled until both fields are filled (AUTH-01 rules).
    let update = glib::clone!(
        #[weak]
        email,
        #[weak]
        password,
        #[weak]
        sign_in,
        move || {
            sign_in.set_sensitive(!email.text().trim().is_empty() && !password.text().is_empty());
        }
    );
    let u = update.clone();
    email.connect_changed(move |_| u());
    password.connect_changed(move |_| update());
    code.connect_changed(glib::clone!(
        #[weak]
        verify,
        move |c| verify.set_sensitive(!c.text().trim().is_empty())
    ));

    let mfa_token: Rc<RefCell<Option<String>>> = Rc::default();

    let finish = {
        let app = app.clone();
        let ctx = ctx.clone();
        let window = window.clone();
        Rc::new(move || {
            let api = ctx.api.clone();
            let (app, ctx, window) = (app.clone(), ctx.clone(), window.clone());
            runtime::spawn(async move { api.capabilities().await }, move |r| match r {
                Ok((me, caps)) => {
                    *ctx.me.borrow_mut() = Some(me);
                    *ctx.caps.borrow_mut() = caps;
                    crate::show_shell(&app, ctx);
                    window.close();
                }
                Err(e) => eprintln!("substratal: couldn't load your permissions: {e}"),
            });
        })
    };

    let submit = {
        let (ctx, nav, mfa_page, mfa_token, finish) = (
            ctx.clone(),
            nav.clone(),
            mfa_page.clone(),
            mfa_token.clone(),
            finish.clone(),
        );
        let (email, password, error, sign_in) = (
            email.clone(),
            password.clone(),
            error.clone(),
            sign_in.clone(),
        );
        move || {
            if !sign_in.is_sensitive() {
                return;
            }
            sign_in.set_sensitive(false);
            email.set_editable(false);
            password.set_editable(false);
            error.set_visible(false);
            let (api, e, p) = (
                ctx.api.clone(),
                email.text().trim().to_string(),
                password.text().to_string(),
            );
            let (nav, mfa_page, mfa_token, finish) = (
                nav.clone(),
                mfa_page.clone(),
                mfa_token.clone(),
                finish.clone(),
            );
            let (email, password, error, sign_in) = (
                email.clone(),
                password.clone(),
                error.clone(),
                sign_in.clone(),
            );
            runtime::spawn(async move { api.login(&e, &p).await }, move |r| {
                email.set_editable(true);
                password.set_editable(true);
                sign_in.set_sensitive(true);
                match r {
                    Ok(LoginOutcome::Session(_)) => finish(),
                    Ok(LoginOutcome::MfaRequired(c)) => {
                        *mfa_token.borrow_mut() = Some(c.mfa_token);
                        nav.push(&mfa_page);
                    }
                    Err(e) => {
                        error.set_label(&error_text(&e));
                        error.set_visible(true);
                        password.set_text("");
                    }
                }
            });
        }
    };
    let s = submit.clone();
    sign_in.connect_clicked(move |_| s());
    let s = submit.clone();
    email.connect_entry_activated(move |_| s());
    password.connect_entry_activated(move |_| submit());

    let verify_now = {
        let (ctx, code, mfa_error, verify, mfa_token, finish) = (
            ctx.clone(),
            code.clone(),
            mfa_error.clone(),
            verify.clone(),
            mfa_token.clone(),
            finish.clone(),
        );
        move || {
            let Some(token) = mfa_token.borrow().clone() else {
                return;
            };
            verify.set_sensitive(false);
            mfa_error.set_visible(false);
            let (api, c) = (ctx.api.clone(), code.text().to_string());
            let (code, mfa_error, verify, finish) = (
                code.clone(),
                mfa_error.clone(),
                verify.clone(),
                finish.clone(),
            );
            runtime::spawn(async move { api.verify_mfa(&token, &c).await }, move |r| {
                verify.set_sensitive(true);
                match r {
                    Ok(_) => finish(),
                    Err(e) => {
                        mfa_error.set_label(&error_text(&e));
                        mfa_error.set_visible(true);
                        code.set_text("");
                    }
                }
            });
        }
    };
    let v = verify_now.clone();
    verify.connect_clicked(move |_| v());
    code.connect_entry_activated(move |_| verify_now());

    window.present();
    email.grab_focus();
}
