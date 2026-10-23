//! `substratalctl`: the Substratal Platform API from a terminal, sharing the
//! Hub's core (same versioning, auth, refresh, errors and permissions).
//!
//! The command definition lives in the library so `xtask` can generate the
//! CLI reference page in the help center from it.

use std::io::{BufRead, Write};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use substratal_core::models::LoginOutcome;
use substratal_core::secrets::SecretStore;
use substratal_core::{ApiClient, Environment, config, screens, version};

#[derive(Parser, Debug)]
#[command(name = "substratalctl", version, about = "Command-line client for the Substratal Apps Platform API", long_about = None)]
pub struct Cli {
    /// API environment: `production`, `local` (the devserver on :8080) or a base URL.
    #[arg(
        long,
        global = true,
        env = "SUBSTRATAL_API",
        default_value = "production"
    )]
    pub api: String,
    /// Don't use the Secret Service; you sign in again each run.
    #[arg(long, global = true)]
    pub no_keyring: bool,
    /// Print JSON instead of a table.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check that the API is up (`GET /health`).
    Health,
    /// Show this client's version, the API version it sends, and the base URL.
    Version,
    /// Sign in with email and password (and a TOTP code if two-factor is on).
    Login {
        #[arg(long)]
        email: String,
        /// Read the password from this environment variable instead of prompting.
        #[arg(long, value_name = "VAR")]
        password_env: Option<String>,
    },
    /// Sign out and forget the saved session.
    Logout,
    /// Show the signed-in user and their platform permissions.
    Whoami,
    /// Manage users.
    #[command(subcommand)]
    Users(UsersCommand),
    /// List the Hub's screens (IDs, routes, who can use them).
    Screens {
        /// Include web-only screens (the Console).
        #[arg(long)]
        all: bool,
    },
    /// Resolve a route or `substratal://` link to its screen.
    Route { path: String },
    /// Print the help-center URL for a screen ID, e.g. `USR-01`.
    HelpUrl { screen_id: String },
}

#[derive(Subcommand, Debug)]
pub enum UsersCommand {
    /// List users (`GET /users`); needs `users.list`.
    List {
        /// Search by name, email or `usr_` ID prefix.
        #[arg(long, short)]
        query: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
}

/// The clap command, for documentation generation.
pub fn command() -> clap::Command {
    <Cli as clap::CommandFactory>::command()
}

pub async fn run(cli: Cli) -> Result<()> {
    let env = Environment::parse(&cli.api);
    match &cli.command {
        Command::Screens { all } => return list_screens(*all, cli.json),
        Command::Route { path } => return route(path),
        Command::HelpUrl { screen_id } => {
            let s = screens::get(screen_id).with_context(|| format!("no screen {screen_id}"))?;
            println!("{}", s.help_url());
            return Ok(());
        }
        Command::Version => {
            println!("substratalctl {}", env!("CARGO_PKG_VERSION"));
            println!(
                "API version   {}: {} (header, not in the URL)",
                version::VERSION_HEADER,
                version::API_VERSION
            );
            println!("API base URL  {}", env.base_url());
            println!("Specs         {}", screens::spec_commit());
            return Ok(());
        }
        _ => {}
    }

    // Only commands that use a session touch the Secret Service.
    let needs_session = !matches!(cli.command, Command::Health);
    let secrets = if cli.no_keyring || !needs_session {
        SecretStore::memory()
    } else {
        SecretStore::system().await
    };
    let ua = format!(
        "Substratal-CLI/{} ({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );
    let api = ApiClient::new(env, secrets, &ua, "en-US")?;

    match cli.command {
        Command::Health => {
            let h = api.health().await?;
            println!("{}", h.status);
        }
        Command::Login {
            email,
            password_env,
        } => {
            let password = match password_env {
                Some(var) => std::env::var(&var).with_context(|| format!("{var} isn't set"))?,
                None => prompt("Password: ")?,
            };
            match api.login(&email, &password).await? {
                LoginOutcome::Session(s) => println!("Signed in as {}", s.user_id),
                LoginOutcome::MfaRequired(c) => {
                    let code = prompt("Two-factor code (or recovery code): ")?;
                    let s = api.verify_mfa(&c.mfa_token, &code).await?;
                    println!("Signed in as {}", s.user_id);
                }
            }
            if !api.session().secrets().is_persistent() {
                eprintln!("Note: no Secret Service, so this session ends when the command exits.");
            }
        }
        Command::Logout => {
            api.restore().await?;
            api.logout().await?;
            println!("Signed out");
        }
        Command::Whoami => {
            ensure_session(&api).await?;
            let (me, caps) = api.capabilities().await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({ "id": me.id, "email": me.email, "permissions": caps.permissions, "owned_apps": caps.owned_apps, "mfa_enabled": me.mfa_enabled })
                );
            } else {
                println!("{}  {}  ({:?})", me.id, me.email, me.status);
                println!(
                    "permissions: {}",
                    if caps.permissions.is_empty() {
                        "none".into()
                    } else {
                        caps.permissions
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
                if !caps.owned_apps.is_empty() {
                    println!("owns apps:   {}", caps.owned_apps.join(", "));
                }
            }
        }
        Command::Users(UsersCommand::List { query, limit }) => {
            ensure_session(&api).await?;
            let page = api.list_users(query.as_deref(), None, limit).await?;
            if cli.json {
                let rows: Vec<_> = page.data.iter().map(|u| serde_json::json!({"id": u.id, "email": u.email, "status": u.status.key()})).collect();
                println!("{}", serde_json::Value::Array(rows));
            } else {
                for u in &page.data {
                    println!("{:<30} {:<10} {}", u.id, u.status.key(), u.email);
                }
                if page.page.has_more {
                    eprintln!("(more results; raise --limit)");
                }
            }
        }
        Command::Screens { .. }
        | Command::Route { .. }
        | Command::HelpUrl { .. }
        | Command::Version => unreachable!(),
    }
    Ok(())
}

async fn ensure_session(api: &ApiClient) -> Result<()> {
    if !api.restore().await? {
        bail!("not signed in; run `substratalctl login --email you@example.com`");
    }
    Ok(())
}

fn prompt(label: &str) -> Result<String> {
    eprint!("{label}");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

fn list_screens(all: bool, json: bool) -> Result<()> {
    let list: Vec<_> = screens::all()
        .iter()
        .filter(|s| all || s.on_linux())
        .collect();
    if json {
        let rows: Vec<_> = list
            .iter()
            .map(|s| serde_json::json!({"id": s.id, "title": s.title, "route": s.route, "archetype": s.archetype, "who": s.who, "platforms": s.platforms, "help": s.help_url()}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for s in list {
            println!("{:<8} {:<34} {}", s.id, s.route, s.title);
        }
    }
    Ok(())
}

fn route(path: &str) -> Result<()> {
    let m = if path.contains("://") {
        screens::match_link(path)
    } else {
        screens::match_route(path, true)
    };
    let m = m.with_context(|| format!("no screen matches {path}"))?;
    let s = screens::get(&m.screen_id).unwrap();
    println!("{} {} ({})", s.id, s.title, s.route);
    for (k, v) in m.params {
        println!("  {k} = {v}");
    }
    println!("  help: {}", config::help_url(&s.id));
    Ok(())
}
