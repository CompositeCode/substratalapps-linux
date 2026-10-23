# Prerequisites check before build-out

**Checked:** 2026-10-23, before the first code in this repo.
**Result:** ready to build. Nothing blocks the skeleton. Items marked ⚠️ limit what can be verified, not what can be built.

## API versioning (read this first)

The Platform API's version is a **request header, not part of the URL**.

- Since API v1.7.0 (`openapi.yaml` 1.0.0-draft.4, 2026-10-21) there is **no `/v1` path prefix**. Endpoints live at the host root: `https://api.substratalapps.com/users`.
- Every request sends `Substratal-Version: 1`. Without it: `400 api_version_required`. With an unknown value: `400 unsupported_api_version`. Both list `details.supported`.
- Old `/v1/...` paths return `404 not_found`, with no redirect.
- Exempt: `GET /health`, `POST /auth/oauth/token`, `/.well-known/*`, `/scim/v2`.
- A deprecated version gets `Deprecation` (RFC 9745) and `Sunset` (RFC 8594) headers, and keeps working for at least 6 months.

**Where this is recorded:**
- the API's Conventions → Versioning page
- the client specs (CLIENT-ARCHITECTURE §4.1, CHANGES §00, OPEN-QUESTIONS #23)
- this repo's code: `crates/substratal-core/src/version.rs` and `config.rs`
- the Hub help center's [API versioning](https://compositecode.github.io/substratalapps-applications-specs/help/developers/api-versioning.html) page

The two plan documents in this folder were written without `/v1` paths. The sibling macOS plan still uses `/v1/ops/...`.

## Verified

| Area | Check | Result |
|---|---|---|
| Live API | `GET https://api.substratalapps.com/health` | `200 {"status":"ok"}` |
| Live API | `GET /v1/health` | `404 not_found`: the prefix is gone |
| Live API | `GET /users`, no header | `400 api_version_required`, `details.supported: ["1"]` |
| Live API | `GET /users`, `Substratal-Version: 1`, no token | `401 unauthenticated`, response echoes `substratal-version: 1` |
| Live API | `POST /auth/login` (one attempt, nonexistent account, via `substratalctl`) | `401 invalid_credentials` with `X-Request-Id`: error envelope and request IDs decode correctly |
| API spec | `origin/main` of `CompositeCode/substratalapps.com` | `openapi.yaml` **1.0.0-draft.9** (API v1.8.0), 124 operations, OpenAPI 3.1; vendored to `vendor/openapi.yaml` |
| API changes since Plan 2 | merged | Plan 2 gaps closed since 2026-10-21: test-user expiry (#58), self-service apps (#60), audit export (#61), SCIM (#62), `GET /users?role_id=` and sign-out-everywhere (#68), `Deprecation`/`Sunset` exposed to browsers (#69) |
| Client specs | `CompositeCode/substratalapps-applications-specs` @ `cebf679` | Targets API v1.8.0. `linux/` has PLATFORM-SPEC, all 75 screens in SCREENS.md, theme CSS (5 light + 5 dark), brand assets, mockups. Synced into `data/spec/` by `scripts/sync-specs.py` |
| Client specs | Console screens (OPS-*, SEC-01, CFG-*, RPT-01, INT-01) | **Web-only for now** (OPEN-QUESTIONS #27), read through a web-Hub Ops gateway rather than an API ops Lambda. This supersedes Plan 1's Linux Operations section. The registry keeps them as `web` so the shared help center documents them. |
| Toolchain (Mac) | Rust 1.96, Docker 20.10, Ruby 3.2 + Jekyll 4.3, Node 25, gh 2.92 | OK. GTK 4 and libadwaita are **not** installed on the Mac, so builds run in the container. |
| Toolchain (Linux) | `build-aux/docker/Dockerfile.dev` (Ubuntu 24.04) | Rust 1.99, **GTK 4.14.5, libadwaita 1.5.0** (the spec's minimums), Blueprint 0.12, Xvfb for headless screenshots. `scripts/dev.sh` wraps it. |
| Flatpak | `ghcr.io/flathub-infra/flatpak-github-actions` tags | gnome-44 … gnome-51 exist; the manifest targets GNOME 50. Built in CI (Flatpak can't run on macOS). |
| GitHub | `CompositeCode/substratalapps-linux` | Was private on the org's Free plan, so GitHub Pages wasn't available. **Made public** (decision 2026-10-23) to publish the help center on Pages, matching the API docs. |
| GitHub | Actions | Available. No secrets required. An optional `SYNC_TOKEN` PAT makes the automated sync PRs trigger CI. |
| AWS | needed for this phase? | No. The app and help site add nothing to AWS. |

## Limits on verification

| Item | Effect | To unblock |
|---|---|---|
| ⚠️ No test account on the live API | Signed-in screens (Users table, Account, Capabilities from real roles) are built and covered by the stub-server integration tests, but haven't run against production data | A test-mode account per persona (superadmin, support, billing admin, app owner, member), or run the API's devserver (`make up && make run` in the API repo) and use `SUBSTRATAL_API=local` |
| ⚠️ No Secret Service in containers | The app falls back to memory-only sessions there and says so on stderr | Nothing to do: real desktops and the Flatpak (Secret portal) have one |
| ⚠️ Generated API client | Plan 1 §6.1 said to spike a generated Rust client. The skeleton uses hand-written models for the operations it calls, plus a shared middleware (`api.rs`) that already applies every request convention. | Spike `progenitor` (after a 3.1 → 3.0 down-convert) or `openapi-generator` when the screens need many more operations |
| ⚠️ Spec icon names | `mail-inbox-symbolic`, `wallet-symbolic` and `bell-symbolic` (INFORMATION-ARCHITECTURE §5) aren't in Adwaita 46+ | The app bundles small originals under those names. Worth raising with the specs team. |
