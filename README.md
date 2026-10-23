# Substratal Hub for Linux

The Substratal Hub for Linux (GNOME): a GTK 4 + libadwaita client of the [Substratal Apps Platform API](https://compositecode.github.io/substratalapps.com/), and the management app for it. Built to the client specs in [substratalapps-applications-specs](https://github.com/CompositeCode/substratalapps-applications-specs) (`linux/`).

**Help:** the Hub help center at https://compositecode.github.io/substratalapps-applications-specs/help/, the one documentation site for every Hub app. It's built from `help/` in the [specs repo](https://github.com/CompositeCode/substratalapps-applications-specs). The Linux pages are [Substratal on Linux](https://compositecode.github.io/substratalapps-applications-specs/help/guides/linux.html), [Keyboard shortcuts on Linux](https://compositecode.github.io/substratalapps-applications-specs/help/guides/linux-keyboard-shortcuts.html) and [substratalctl](https://compositecode.github.io/substratalapps-applications-specs/help/developers/substratalctl.html). The app's Help button and F1 open `help/screens/<id>.html` for the current screen.

> **API versioning.** The API version is the `Substratal-Version: 1` request header. URLs have no `/v1` (`https://api.substratalapps.com/users`), and `/v1/...` returns `404`. The version is set in one place, `crates/substratal-core/src/version.rs`, and sent on every request by the client middleware.

## Layout

| Path | What |
|---|---|
| `crates/substratal-core` | GTK-free core: API client (versioning, auth and single-flight refresh, idempotency, ETags, retries, error decoding), Secret Service storage, Capabilities, the screen registry, navigation, shortcuts |
| `crates/substratal-hub` | The GTK 4 / libadwaita app: window shell, sidebar, router and `substratal://` links, sign-in (Blueprint), themes, the spec-driven screen scaffold, and the hand-built screens |
| `crates/substratalctl` | Command-line companion on the same core |
| `crates/xtask` | `cargo xtask gen-help --out <specs>/help`: generates the help center's Linux pages (shortcuts, CLI reference) from this code |
| `data/spec/` | Generated from the specs package by `scripts/sync-specs.py`. Never edit by hand. |
| `vendor/openapi.yaml` | The Platform API spec this build targets, kept current by the API spec sync workflow |
| `build-aux/` | Flatpak manifest, Linux dev container |
| `planning/` | Plans, the API-gap list and the prerequisites check |

## Build and run

On Linux with GTK 4.14+, libadwaita 1.5+ and `blueprint-compiler`:

```sh
cargo run -p substratal-hub                         # the app (production API)
SUBSTRATAL_API=local cargo run -p substratal-hub    # against the API's devserver on :8080
SUBSTRATAL_PREVIEW=superadmin cargo run -p substratal-hub   # layout preview as a persona, no sign-in
cargo run -p substratalctl -- health
```

On macOS, or anywhere without GTK, use the container:

```sh
scripts/dev.sh cargo test --workspace
scripts/dev.sh scripts/screenshot.sh _screens / /admin/users /settings   # headless screenshots
```

## Checks (what CI runs)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/sync-specs.py --check        # spec-derived files match the specs package
```

## Automation

| Workflow | When | Does |
|---|---|---|
| `ci.yml` | PRs, pushes to `main` | Build, lint, test, spec and help-page checks, desktop/AppStream validation |
| `docs-dispatch.yml` | Pushes to `main` touching the app | Regenerates the Linux help pages, opens a PR in the specs repo, and sends `app-docs-updated` so the help center rebuilds (needs `SPECS_DOCS_TOKEN`) |
| `api-spec-sync.yml` | Every 6 h, on demand, `repository_dispatch: api-spec-updated` | Pulls the API's `openapi.yaml`, diffs it with oasdiff, opens a PR (labelled when breaking) |
| `specs-sync.yml` | Daily, on demand | Pulls the client specs, regenerates screens, themes and icons, opens a PR |
| `flatpak.yml` | Pushes to `main`, on demand | Builds Flatpak bundles (x86_64, aarch64) as artifacts |

## License

MIT
