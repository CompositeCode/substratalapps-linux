# Substratal Hub for Linux

The Substratal Hub for Linux (GNOME): a GTK 4 + libadwaita client of the [Substratal Apps Platform API](https://compositecode.github.io/substratalapps.com/), and the management app for it. Built to the client specs in [substratalapps-applications-specs](https://github.com/CompositeCode/substratalapps-applications-specs) (`linux/`).

**Help center:** https://compositecode.github.io/substratalapps-linux/. It's built from `docs/` in this repo and serves every Hub app (Linux, macOS, Windows, iOS/iPadOS, Android, Web).

> **API versioning.** The API version is the `Substratal-Version: 1` request header. URLs have no `/v1` (`https://api.substratalapps.com/users`), and `/v1/...` returns `404`. The version is set in one place, `crates/substratal-core/src/version.rs`, and sent on every request by the client middleware.

## Layout

| Path | What |
|---|---|
| `crates/substratal-core` | GTK-free core: API client (versioning, auth and single-flight refresh, idempotency, ETags, retries, error decoding), Secret Service storage, Capabilities, the screen registry, navigation, shortcuts |
| `crates/substratal-hub` | The GTK 4 / libadwaita app: window shell, sidebar, router and `substratal://` links, sign-in (Blueprint), themes, the spec-driven screen scaffold, and the hand-built screens |
| `crates/substratalctl` | Command-line companion on the same core |
| `crates/xtask` | `cargo xtask gen-docs`: generates the help center's reference pages from the code, the specs and the API spec |
| `data/spec/` | Generated from the specs package by `scripts/sync-specs.py`. Never edit by hand. |
| `vendor/openapi.yaml` | The Platform API spec this build targets, kept current by the API spec sync workflow |
| `docs/` | The help center (Jekyll + just-the-docs), versioned and deployed to GitHub Pages |
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
cargo xtask gen-docs --check                 # generated help pages match the code
python3 scripts/sync-specs.py --check        # spec-derived files match the specs package
```

## Automation

| Workflow | When | Does |
|---|---|---|
| `ci.yml` | PRs, pushes to `main` | Build, lint, test, generated-docs and spec checks, desktop/AppStream validation, site build |
| `docs.yml` | Pushes to `main` touching app, docs, data or spec | Regenerates reference pages, builds a new docs version, deploys to GitHub Pages |
| `api-spec-sync.yml` | Every 6 h, on demand, `repository_dispatch: api-spec-updated` | Pulls the API's `openapi.yaml`, diffs it with oasdiff, updates the API changelog and pages, opens a PR (labelled when breaking) |
| `specs-sync.yml` | Daily, on demand | Pulls the client specs, regenerates screens, themes and help pages, opens a PR |
| `flatpak.yml` | Pushes to `main`, on demand | Builds Flatpak bundles (x86_64, aarch64) as artifacts |

## License

MIT
