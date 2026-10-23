# Plan 1: Substratal Console for Linux (management app for the API tier)

> **Update, 2026-10-23.** Two things changed after this plan was written. See [PREREQUISITES.md](PREREQUISITES.md).
> 1. **API versioning:** the API version is the `Substratal-Version: 1` request header, and URLs have no `/v1` prefix (API v1.7.0+). This plan already uses unprefixed paths. The macOS plan's `/v1/ops/*` should read `/ops/*`.
> 2. **The specs now make the Console web-only** (OPEN-QUESTIONS #27): its OPS-*, SEC-01, CFG-*, RPT-01 and INT-01 screens read AWS through the web Hub's Ops gateway, and the native apps skip them. So the Linux build follows the spec's 75 Hub screens, and §3's Operations section and §4.5–4.11 are **deferred** for Linux. The screen IDs in this plan (OPS-01…10) also differ from the spec's. The spec's IDs win.
>
> Since 2026-10-21, the API has also shipped several Plan 2 items: `GET /users?role_id=`, sign out everywhere, audit export, self-service registration, SCIM and test-user expiry.


**Status:** proposal, 2026-10-21
**Target:** `https://api.substratalapps.com`, with every request sending `Substratal-Version: 1`. Live on 2026-10-21: `GET /health` returns `200 {"status":"ok"}`. The old `/v1/...` paths now return `404`.
**Inputs reviewed:**
- API repo `CompositeCode/substratalapps.com` at `origin/main` `364ce5b` (2026-10-21): `openapi.yaml` 1.0.0-draft.4 (118 operations), `plan-remaining.md`, `infra/terraform/prod`, `internal/core`, and the four open PRs #58, #60, #61 and #62.
- Specs repo `CompositeCode/substratalapps-applications-specs`: `linux/PLATFORM-SPEC.md`, `linux/SCREENS.md`, `linux/theme/` and `00-shared/*`.
- The sibling macOS plan (`substratalapps-macos/docs/`), so that screen IDs and API asks stay the same on every platform.

**Companion:** [PLAN-api-gaps.md](PLAN-api-gaps.md) covers what the API needs before this app can do everything below. Section references like "Plan 2 §3" point there.

---

## 0. Summary

- **Can we start building now?** Yes, for roughly 70% of the app. The API already exposes and implements everything about the *objects* it manages: users, sessions, access (entitlements), roles, applications, API keys, webhooks, push credentials, tenants, organizations, billing and audit. Those screens are unblocked today.
- **What's missing** is everything about the *system itself*: health beyond a shallow `ok`, usage and performance metrics, logs, alarms, runtime configuration and feature flags, job and queue visibility, cost, and application retirement. These live only in CloudWatch, Terraform and Go constants. Plan 2 covers them.
- **Shape:** build the Linux **Substratal Hub** the specs already define (GTK 4 + libadwaita, same 75 screens, routes and themes), **admin-first**. Add one new sidebar section, **Operations** (OPS-01…10, the same IDs as macOS). One more Linux-specific piece: the Rust core is also shipped as a CLI, **`substratalctl`**, for terminals, SSH sessions, CI and AWS CloudShell.
- **Stack:** Rust · gtk4-rs + libadwaita-rs · Blueprint · a client generated from `openapi.yaml` · `oo7` (Secret Service) · SQLite cache · Flatpak. Distribution is a **self-hosted Flatpak repo on the existing S3 + CloudFront** (Flathub comes later, for the end-user Hub).
- **New AWS cost:** about $0 for the app itself. The API side (Plan 2) adds one `ops` Lambda, for roughly **$2–4/month** in total. There are no servers, VPC, NAT, Grafana or OpenSearch.

---

## 1. Positioning: Console vs. Hub

| Option | Verdict |
|---|---|
| A. A separate admin-only "Console" app | It would duplicate sign-in, session refresh, Capabilities, theming and the whole Administration and Developer screen set the spec already defines. It would also need a second Flatpak app ID and a second `x-scheme-handler`. |
| **B. The Hub, built admin-first, plus Operations** ✅ | One codebase and one app ID (`com.substratalapps.Hub`). The spec already hides Administration and Developer from people without rights. Operations is gated by a new `ops.view` permission (Plan 2 §9), so end users never see it. The Personal screens come last in the build order (§8). |

"Console" below means the Operations, Administration and Developer sections of the Linux Hub.

---

## 2. Personas

| Persona | Platform permissions (API catalog, 13 today) | Console sections |
|---|---|---|
| **Superadmin** (protected seed role) | all 13, plus `ops.*` once added | everything |
| **Operator / SRE** *(new)* | `ops.view`, `ops.manage` *(Plan 2 §9)*, `audit.view`, `users.list` | Operations, Audit, Users (read) |
| **Support** | `users.list`, `entitlements.manage`, `audit.view`, `tenants.manage`, `billing.manage` | Users, Access, Audit, Tenant, user notifications |
| **Billing admin** | `billing.manage`, `billing.refund`, `audit.view`, `users.list` | Users (read), billing views, Audit |
| **App owner (developer)** | none; owns ≥ 1 Application | Applications (own), API Keys (own app), Webhooks (own app), Access (confined to own app), Usage (own app, Plan 2 §2) |

The admin model maps directly onto the API's own users, roles and permissions. **No AWS IAM identities are involved.** The Console never holds AWS credentials or uses an API key. Every action runs as a User session, so audit events always show `actor_type: user`. The API also enforces this: API keys can't manage keys (`400 user_token_required`).

---

## 3. Information architecture

This extends `00-shared/INFORMATION-ARCHITECTURE.md`. The PERSONAL, ADMINISTRATION and DEVELOPER sections are unchanged. **OPERATIONS** is new.

```
Substratal Hub (Linux)
├── PERSONAL            Home · Inbox · Organizations · Account · Plan & Billing · Settings      (spec, unchanged)
├── OPERATIONS  (new)   /ops                           visible with ops.view; flag `operations`
│   ├── Overview        /ops                OPS-01  system status board
│   ├── Usage           /ops/usage          OPS-02  traffic and analytics by app / key / route / status
│   ├── Performance     /ops/performance    OPS-03  latency, errors, cold starts, DB capacity
│   ├── Logs            /ops/logs           OPS-04  request and security log search
│   ├── Alerts          /ops/alerts         OPS-05  alarms, rules, routing  (+ OPS-05b rule editor)
│   ├── Jobs & Queues   /ops/jobs           OPS-06  11 scheduled jobs, webhook/push queues, DLQ
│   ├── Configuration   /ops/config         OPS-07  flags, rate limits, sign-up, security, maintenance (+ OPS-07b history)
│   ├── Integrations    /ops/integrations   OPS-08  Slack / PagerDuty / email routes, platform webhooks, MCP
│   ├── Cost            /ops/cost           OPS-09  AWS spend vs. warn/hard budgets
│   └── Releases        /ops/releases       OPS-10  deployed commit, migrations, API/docs version
├── ADMINISTRATION      Users · Access · Roles · Audit Log · Send notification                 (spec D6–D8, D12, NTF-06)
└── DEVELOPER           Applications · API Keys · Webhooks · Tenant                             (spec D9–D11)
                        + APL-08 Retire application · KEY-05 Key usage tab                     (new)
```

- **Sidebar order:** Personal, **Operations**, Administration, Developer. That's the same order as macOS. Operations goes high because it's what an operator opens the app for.
- **Gating:** Operations shows when the person holds `ops.view`. Until the API serves `/ops/*`, the `operations` flag is off. A `404` or `501` from `/ops/*` turns it off for the session, following the spec's existing rule for flag-gated features.
- **Routes and deep links:** `substratal://ops/alerts/{alarm}` and the rest, through the existing `x-scheme-handler/substratal` and `DBusActivatable=true` (so a link reaches the running instance).
- **Keyboard** (added to the spec's shortcut table and the `AdwShortcutsDialog`): **Ctrl+Shift+1 … Ctrl+Shift+9** for Overview … Cost. **Ctrl+L** focuses the Logs query bar. **Ctrl+Shift+T** opens the shared time-range picker.
- **Adwaita symbolic icons:** Overview `speedometer-symbolic` (or `utilities-system-monitor-symbolic`) · Usage `view-statistics-symbolic` · Performance `power-profile-performance-symbolic` · Logs `system-search-symbolic` · Alerts `dialog-warning-symbolic` · Jobs `alarm-symbolic` · Configuration `preferences-other-symbolic` · Integrations `application-x-addon-symbolic` · Cost `money-symbolic` (bundled if the theme lacks it) · Releases `software-update-available-symbolic`. Each needs adding to INFORMATION-ARCHITECTURE §5 for all six platforms.

---

## 4. Feature set

Each feature lists its screens, the API it uses, and a readiness flag:
**✅ Ready** (the API exists today) · **🟡 Partial** (works with a workaround; Plan 2 improves it) · **🔵 In review** (an open API PR adds it) · **🔴 Blocked** (needs Plan 2).

### 4.1 Sign-in and admin session security

| Feature | Screens | API | Ready |
|---|---|---|---|
| Email + password sign-in, TOTP and recovery-code challenge, in its own `AdwWindow` before the main window | AUTH-01/02 | `POST /auth/login`, `/auth/mfa/verify` | ✅ |
| Single-flight refresh (a `tokio::sync::Mutex`-guarded `SessionStore`). The rotated refresh token is written to Secret Service *before* the new access token is used. Proactive refresh 60 s before expiry. | — | `/auth/token/refresh`, `/auth/logout` | ✅ |
| Idle auto-lock after 15 min for anyone with a platform permission, plus a lock when the GNOME session locks (`org.freedesktop.ScreenSaver` / logind `Lock` signal). Unlock with the account password; a TOTP code is accepted if MFA is on. | GLB-03 | local | ✅ |
| Own sessions list and "sign out other sessions" | ACC-04 | `/users/me/sessions` | ✅ |
| **Require MFA before admin sections open**: the Console blocks Operations, Administration and Developer until TOTP is enrolled | ACC-05 | client-side now; server-side in Plan 2 §4 | 🟡 |
| Step-up re-authentication on destructive actions (suspend user, revoke key, rotate secrets, delete role, retire app, run job, maintenance mode) | `AdwAlertDialog` with a TOTP entry row | Plan 2 §4 `POST /auth/step-up` | 🔴 (client-side re-prompt meanwhile) |
| SSO and passkeys for staff | AUTH-01 | SSO returns `501` | 🔴 flag off |

### 4.2 User authentication management: users, roles, permissions

| Feature | Screens | API | Ready |
|---|---|---|---|
| Directory in a `GtkColumnView` (virtualized, sortable, multi-select): search `q`/`email`, filter by status / app / org / created range; inspector in an end-side `AdwOverlaySplitView` | USR-01 | `GET /users` | ✅ |
| Invite, resend invitation, create test-mode users | USR-02 | `POST /users`, `/users/{id}/invitation` | ✅ (expiry 🔵 PR #58) |
| Overview, edit, suspend/reactivate, delete, data export (save with `GtkFileDialog`), erasure request/cancel | USR-03/05 | `/users/{id}`, `/suspend`, `/export`, `/erasure-requests` | ✅ |
| Per-user sessions: list (UA, IP, last seen) and revoke | USR-04 | `/users/{id}/sessions/{sessionId}` | ✅ |
| "Sign out everywhere" for one user | USR-04 | loop over sessions, 4 in parallel | 🟡 (Plan 2 §4 adds one call) |
| Reset a user's MFA (lost device) | USR-03 | `DELETE /users/{id}/mfa/totp` (`user.mfa_reset`) | ✅ |
| Roles: list, detail, new, edit the permission checklist (only permissions the editor holds are enabled, which mirrors the API's no-escalation rule), delete; seed roles are read-only | ROL-01..03 | `/roles`, `/permissions` | ✅ |
| Assign and remove roles per user | ROL-04 | `/users/{id}/roles/{roleId}` | ✅ |
| "People with this role" | ROL-02 | needs `GET /users?role_id=` | 🔴 hidden |
| Bulk suspend, grant or assign | USR-01, ACX-05 | one request per row, max 4 in flight, per-row results | 🟡 |

### 4.3 Application management (add, configure, review, remove)

| Feature | Screens | API | Ready |
|---|---|---|---|
| List all applications, filter by owner, `review_status`, `access_tier` | APL-01 | `GET /applications` | ✅ |
| **Add**: register an application. Platform admins today; self-service for developers lands with PR #60. | APL-02 | `POST /applications` | ✅ / 🔵 |
| Overview and edit: launch URL, icon URL, visibility, default/available app roles, `receives_platform_notifications` | APL-03 | `GET/PATCH /applications/{id}` (`If-Match`) | ✅ |
| Settings schema editor: drop a JSON file from Files, validate, show stale-override counts | APL-04 | `PATCH settings_schema` | ✅ |
| App permissions and app-scoped roles | APL-05 | `/applications/{id}`, `/roles?scope=` | ✅ |
| **Review queue**: approve, reject (notes required), **suspend** (the kill switch) | APL-06 | `PATCH review_status`, `?review_status=pending_review` | ✅ |
| Push credentials: APNs `.p8`, FCM JSON, VAPID. The key file is read from the picked file into memory, sent once, and never cached. | APL-07 | `/applications/{id}/push-credentials` | ✅ |
| Who has access to this app, plus its usage and activity | APL-03 tabs | `GET /users?application_id=`, `/audit-events?application_id=`, usage (Plan 2 §2) | ✅ / 🔴 |
| Push health for this app (sent / failed / invalid tokens) | APL-07 | `notification.push_failed` / `push_credential.failed` webhooks only; no read endpoint | 🔴 Plan 2 §3 |
| **Remove: retire or delete an application** | APL-08 (new), from the APL-03 ⋯ menu | **No DELETE or retire endpoint exists.** Suspend is the only off switch, and it's reversible. | 🔴 Plan 2 §6 |

### 4.4 API key management (clients and internal use)

The API's key model is already admin-grade. It has platform and app scopes, `live`/`test` modes, `intended_use: service | agent`, `restrict_destructive`, `expires_at`, `secret_hint`, `last_used_at`, rotation and soft revoke.

| Feature | Screens | API | Ready |
|---|---|---|---|
| Key list (scope, mode, intended use, include revoked; columns for hint, last used, expires, creator) | KEY-01 | `GET /api-keys` | ✅ |
| Create a key. The permission picker offers only permissions the creator holds, and the agent preset turns on `restrict_destructive`. | KEY-02 | `POST /api-keys` (Idempotency-Key) | ✅ |
| **Secret reveal, exactly once.** It shows in an `AdwDialog` with `can-close=false` until the admin acknowledges it. Copy writes to `GdkClipboard`, which is cleared after 60 s if it still holds the secret. The secret is never logged, cached or put in a notification. | KEY-03 | create/rotate response | ✅ |
| Detail: rename, permissions, `restrict_destructive`; rotate; revoke | KEY-04 | `PATCH`, `/rotate`, `DELETE` | ✅ |
| **Hygiene filter chips:** never used, unused for 30+ days, expiring within 14 days, `service` keys holding destructive permissions, platform keys with broad permissions | KEY-01 | derived client-side | ✅ |
| Per-key request volume, errors and 429s | KEY-05 (new tab) | Plan 2 §2 | 🔴 |
| *Phase 3:* "Deliver to AWS Secrets Manager" for **internal** keys, so the secret never reaches a clipboard | KEY-03 | Plan 2 §8 | 🔴 |

**How the client side handles keys:** the Console manages keys but never stores or uses one. Developers' backends keep their own keys in their own secret stores. Substratal's internal consumers (the Stripe handler, the smoke test, the MCP tests) read theirs from Secrets Manager or GitHub secrets, as they do today. `substratalctl` authenticates as a user too (device-style login, refresh token in Secret Service). It also accepts `SUBSTRATAL_API_KEY` for CI scripts, and in that case the API's own key rules apply.

### 4.5 Monitoring dashboard: OPS-01 Overview

A responsive grid of tiles (`GtkFlowBox` of `.card`s, 1–4 columns by `AdwBreakpoint`). It refreshes every 30 s while the window is mapped and focused, slows to 120 s when unfocused, and stops when hidden.

| Tile | Data | Ready |
|---|---|---|
| API status (shallow), version header echo, round-trip latency measured by the client | `GET /health` | ✅ |
| Component health: Aurora/Data API, KMS, SQS, SES, Stripe, APNs, FCM, Web Push, jobs | `GET /ops/health` | 🔴 |
| Requests, 4xx, 5xx and 429s for the last hour, with sparklines | `GET /ops/overview` | 🔴 |
| p50/p95 latency vs. the 150 ms NFR target for hot paths | same | 🔴 |
| Aurora ACU vs. max; Lambda errors and throttles per function | same | 🔴 |
| Open alarms (13 exist today: API 5xx, Lambda errors ×5, throttles ×2, Aurora near max, webhook DLQ, 3 security filters) | `GET /ops/alarms` | 🔴 |
| Webhook queue depth, DLQ count, oldest pending delivery | `GET /ops/queues` | 🔴 |
| Security signals: failed logins, refresh-token reuse, destructive-blocked | `GET /ops/metrics?set=security` | 🔴 |
| Month-to-date cost vs. the $75 warn and $150 hard budgets | `GET /ops/cost` | 🔴 |
| Recent admin activity (last 20) | `GET /audit-events?limit=20` | ✅ |
| Needs attention: apps pending review, tier-change requests, keys expiring | `/applications?review_status=pending_review`, `/tenants/{id}/tier-change-requests`, `/api-keys` | ✅ |

The board ships early with the ✅ tiles. The 🔴 tiles render an `AdwStatusPage`-style "Not available yet" placeholder until Plan 2 lands.

### 4.6 Usage, analytics and reporting: OPS-02 Usage, OPS-03 Performance

- **Usage:** a time range (1 h / 24 h / 7 d / 30 d / custom) and group-by (application · API key · route · status class · principal type · test/live). Charts are stacked bars plus a top-N `GtkColumnView` (top keys, top apps, top 4xx routes, top 429 callers). Activating a row drills into the key, app or route with the chart filtered.
- **Performance:** latency percentiles per route, error rate, Lambda duration and cold starts per function (api, mcp, stripe-webhook, webhook-worker, jobs), and Aurora ACU and Data API errors.
- **Reports:** export CSV or JSON through `GtkFileDialog`. **Drag a chart or table out as a file** (`GdkFileList`, already a spec interaction). "Copy as image" renders the chart to a PNG texture. The weekly email summary runs server-side (Plan 2 §5). The desktop app doesn't schedule reports.
- **App owners** see the same screen confined to their own apps, as the Usage tab on APL-03.
- **Readiness:** 🔴. The API Gateway access log has no principal, key, app or route template, and nothing aggregates it (Plan 2 §2).

### 4.7 Logging: OPS-04 Logs

- A query bar with structured filter chips: request id, status, route, principal (user or key), application, IP and level, plus a time range. Results go into a virtualized `GtkColumnView` and load more on scroll. Selecting a row opens the inspector with the JSON pretty-printed in a monospace `GtkSourceView`, plus links to the user, key, app, and audit events sharing the `request_id`.
- **Presets:** *Security log* (`msg = "security"`: logins, failed logins, MFA challenges, refresh-token reuse, lockouts, 401/403, rate-limit hits), *5xx last hour*, *429 by key*, *Slow requests > 1 s*. Saved queries are stored per user in GSettings.
- **Pasting a request id** from any error dialog (the spec's "Details" disclosure shows it) jumps here pre-filtered. This closes the loop for support work.
- The server builds every query. The app sends filters, never raw Logs Insights text. 🔴 Plan 2 §3.

### 4.8 Alerts and notifications: OPS-05 Alerts

- **Alarms list:** state, reason, threshold, history, and mute/snooze.
- **Rules (OPS-05b):** edit thresholds only for the whitelisted user-tunable alarms (5xx rate, p95 latency, 429 rate, DLQ depth, budget %). Choose routing per severity: inbox and push, email, Slack, PagerDuty.
- **Delivery to a Linux admin:** ops alarms become **platform notifications** in the API's existing inbox (Plan 2 §5). Linux has no push provider (spec OPEN-QUESTIONS #2), so the app polls `unread-count` every 60 s while running. With the Background portal (opt-in, autostart), it keeps polling with no window open, and the poll drops to 30 s for `ops.view` holders while an alarm is open. Alarms raise a `GNotification` at `HIGH` priority (`URGENT` for `security.*`), with **Acknowledge** and **Open in Console** buttons routed through `app.ack-alarm` and `app.open-route` GActions.
- **Readiness:** 🔴 Plan 2 §5. Today the SNS topic emails one `alarm_email` address and nothing else.

### 4.9 Jobs and queues: OPS-06

- The 11 scheduled jobs in Terraform, each with schedule, last run, duration, result, error and next run, plus **Run now** (step-up): `entitlement-sweep` (5 min), `webhook-dispatch` (1 min), `stripe-events` (10 min), `erasure-cascade`, `webhook-health`, `subscription-access` (hourly), `seat-sync`, `cleanup`, `audit-archive`, `settings-projections`, `push-maintenance` (daily).
- Webhook queue and DLQ depth, oldest message age, and **Redrive DLQ**.
- Failing deliveries across all webhooks: 🟡 the app loops over `GET /webhooks/{id}/deliveries?status=failed` until Plan 2 §3 adds a global query. Per-delivery **Redeliver** works today ✅.
- Jobs and queues are 🔴 (Plan 2 §3, §8).

### 4.10 Configuration settings: OPS-07

Runtime configuration that today requires a deploy:

| Group | Examples | Today |
|---|---|---|
| Feature flags | `organizations`, `notifications`, `push`, `billing`, `sso`, `passkeys`, `webhooks`, `app_registration_self_service`, `audit_export`, `operations` | **none**. The client spec's "remote config" doesn't exist on the server. |
| Rate limits | default 100/min per principal; per-operation overrides (`users.create` 20/min, `notifications.send` 600/min…); per-key overrides | Go constants in `internal/core/ratelimit.go` and the API Gateway stage throttle |
| Sign-up policy | open / invite-only / closed; allowed staff email domains | env |
| Security policy | require MFA for platform roles, admin idle timeout, admin IP allowlist | not implemented |
| Maintenance mode | off / read-only / full, with a message and `Retry-After` | not implemented |
| Runtime (read-only) | region, CORS origins, email sender, push backend, stage throttle, deployed version | env and Terraform |

**UI:** `AdwPreferencesPage`s per group, using `AdwSwitchRow`, `AdwSpinRow`, `AdwComboRow` and `AdwEntryRow` inside an `AdwClamp` (760 sp max). OPS-07b is a history pane with a diff view and **Revert to this version**. Every save sends `If-Match`, shows a diff confirmation, and requires step-up for security and maintenance changes. 🔴 Plan 2 §1.

### 4.11 Integrations with external tools: OPS-08

- **Platform webhooks** (`scope: platform`) work today ✅ (WHK-01..04: create, rotate secret, test, deliveries, redeliver). With Plan 2 §5's `ops.*` event types, any external tool (a status page, Slack bridge or SIEM) can subscribe to alarms, config changes and releases.
- **Built-in alert routes:** a Slack incoming webhook, PagerDuty Events v2, and email. They're configured here but stored and delivered server-side, so their secrets never live on the laptop. 🔴 Plan 2 §5.
- **MCP server:** shows the `/mcp` endpoint, the agent-intended keys (`/api-keys?intended_use=agent`) and the docs link ✅.
- **SCIM** (per organization, PR #62 🔵): issue or rotate an org's SCIM token from ORG-08, with a one-time reveal like KEY-03.
- **Stripe:** read-only health (last webhook received, failures) via `/ops/health` 🔴. Refunds stay in the Stripe dashboard: `billing.refund` has no endpoint by design.
- **GitHub:** links to the deploy and rollback workflows and the `[bespoke]` issues, read-only. The app never stores a GitHub token.

### 4.12 User activity tracking and audit logs: AUD-01/02, Activity tabs

| Feature | API | Ready |
|---|---|---|
| Audit table with filters (actor user/key/system, action from 56 actions, target type/id, user, app, org, tenant, time); inspector with before/after JSON | `GET /audit-events`, `/audit-events/{id}` | ✅ |
| Archived-range banner from `Audit-Hot-Window-Start` | response header | ✅ |
| Per-user **Activity** tab: events by the user plus events about them, merged | two `GET /audit-events` calls | ✅ |
| "My admin activity" | `actor_id=me` | ✅ |
| Drag an event out as `.json` | client | ✅ |
| **Export a range** (JSONL/CSV) | `POST/GET /audit-exports` (PR #61) | 🔵 |
| Free-text search; actions per day per actor charts | Plan 2 §7 | 🔴 |
| Audit of sensitive non-writes: data exports, admin session revokes, staff notification sends, admin sign-ins | not in the `AuditAction` enum today | 🔴 Plan 2 §4 |

### 4.13 Tenants, organizations, billing and notifications (admin side)

All of these are in the spec and ✅ ready:
- Tenants and tier-change requests (BIL-04)
- Organizations, members and seat grants (ORG-*)
- Per-user and per-org subscription views with the Stripe portal hand-off (BIL-01..03)
- Staff "Send notification" (NTF-06)
- A user's notifications and devices (NTF-05)

---

## 5. Interface design

The Console follows `linux/PLATFORM-SPEC.md` exactly:
- `AdwApplicationWindow` (1280×800 default, 360×294 minimum), with an `AdwNavigationSplitView` sidebar and content
- nested split views for list-detail, and `AdwOverlaySplitView` for inspectors
- libadwaita style classes only, with colours only from theme roles
- the 5 light and 5 dark themes from `linux/theme/`, loaded into one application-priority `GtkCssProvider`

### 5.1 OPS-01 Overview (wide, ≥ 900 sp)

```
┌ Substratal ─────────── ☰ ┬ Overview ─────────────── Last 1 h ▾  ⟳  ● All systems normal ─ ─ □ ✕ ┐
│ 🔍 Search  Ctrl+K        │                                                                      │
│ PERSONAL                 │ ╭ API ────────╮ ╭ Requests ─────╮ ╭ Errors ───────╮ ╭ p95 latency ─╮ │
│   Home                   │ │ ● Healthy   │ │ 12.4k ▁▂▃▅▃▂  │ │ 5xx  0.02 %   │ │ 84 / 150 ms   │ │
│   Inbox              3   │ │ v1 · 82 ms  │ │ +8 % vs prev  │ │ 429  31       │ │ ▁▁▂▁▃▁        │ │
│ OPERATIONS               │ ╰─────────────╯ ╰───────────────╯ ╰───────────────╯ ╰───────────────╯ │
│ ▌ Overview               │ ╭ Components ───────────────────────╮ ╭ Open alarms ──────────────────╮ │
│   Usage                  │ │ Aurora ● 0.5 / 4 ACU   Data API ● │ │ ⚠ webhooks-dlq       12 min   │ │
│   Performance            │ │ SQS ● depth 3   KMS ●   SES ● sbx │ │ ⚠ security-failed-logins      │ │
│   Logs                   │ │ Stripe ●  APNs ●  FCM ●  WebPush ●│ │                  View All ›   │ │
│   Alerts             2   │ ╰───────────────────────────────────╯ ╰───────────────────────────────╯ │
│   Jobs & Queues          │ ╭ Needs attention ──────────────────╮ ╭ Cost (month to date) ─────────╮ │
│   Configuration          │ │ 2 apps pending review   Review ›  │ │ $38.12 of $75 warn  ▓▓▓▓░░░   │ │
│   Integrations           │ │ 1 tier-change request   Open ›    │ ╰───────────────────────────────╯ │
│   Cost · Releases        │ │ 3 keys expire ≤ 14 days Keys ›    │ ╭ Recent admin activity ────────╮ │
│ ADMINISTRATION           │ ╰───────────────────────────────────╯ │ dana  api_key.rotated     2 m │ │
│   Users · Access · …     │                                       ╰───────────────────────────────╯ │
│ DEVELOPER                │                                                                      │
│   Applications · Keys …  │                                                                      │
└──────────────────────────┴──────────────────────────────────────────────────────────────────────┘
```

At ≤ 900 sp the grid drops to 2 columns. At ≤ 600 sp the sidebar becomes the first page and tiles stack in one column. The time range moves from the header bar into a bottom `GtkActionBar`.

### 5.2 OPS-04 Logs (wide)

```
┌ Logs ─────────────────────────────────────────── Last 1 h ▾  Presets ▾  ⟳ ┐
│ [status:5xx ✕] [app:app_notes ✕] [+ Filter]       🔍 request id / text    │
├───────────────────────────────────────────────┬───────────────────────────┤
│ Time      Status  Route              ms  Prin. │ req_01M5GW… · 502         │
│ 20:41:03  502     PATCH /entitlem…  812  key_… │ { "msg": "request",       │
│ 20:40:57  500     POST /users/{id}… 304  usr_… │   "op": "entitlements…",  │
│ 20:40:12  429     GET /users        12   key_… │   … }                     │
│ …  (virtualized, loads more on scroll)         │ Open user · Open key ·    │
│                                                │ Audit for this request ›  │
└───────────────────────────────────────────────┴───────────────────────────┘
```

### 5.3 Patterns

- **Status:** extend `status-semantics.json` with `healthy / degraded / down / unknown` and `ok / alarm / insufficient_data / muted`. These map to libadwaita `success / warning / error / dim-label`, always with an icon and text, never colour alone.
- **Charts:** GTK has no chart widget. Use a small in-house `ChartView` (`GtkWidget` subclass drawing with `GtkSnapshot` for sparklines and bars, plus `plotters` with the cairo backend for axes and multi-series lines). Series colours come from new theme roles `chart_1…6`, which must be added to `THEMES.md` and all 10 Linux theme CSS files. Hover shows a `GtkPopover` tooltip, and Ctrl+C copies the underlying rows as CSV. **Accessibility:** each chart exposes an accessible description summarizing min, max and last values, and a **Show as table** toggle sits next to every chart.
- **Time range** is one shared control encoded in the route (`/ops/usage?range=24h&group=key`), so windows restore and links share state.
- **Destructive ops actions** (Run job, Redrive DLQ, Maintenance on, Revoke key, Suspend or retire app) use the spec's confirm pattern: an `AdwAlertDialog` that names the object, a `.destructive-action` response, and a TOTP `AdwEntryRow` once step-up exists.
- **Undo** (`AdwToast` with 10 s Undo and Ctrl+Z) applies to the spec's access toggles. It does not apply to ops actions, which are confirm-first.
- **Live data is polled, never streamed.** API Gateway's HTTP API can't stream Lambda responses cheaply, and polling one aggregated `/ops/overview` every 30 s is cheaper and simpler than WebSockets.
- **Multi-window:** "Open in New Window" from any row's context menu, so an operator can keep Overview on one monitor and Logs on another. All windows share one `ApiClient`, so polls are coalesced, not duplicated.
- **Admin data never persists:** Users, Audit, Logs and other people's notifications are memory-only, per the spec. The window title drops personal data while locked.

### 5.4 New screen IDs (add to the specs repo before build)

OPS-01 … OPS-10, OPS-05b, OPS-07b, APL-08 Retire application, KEY-05 Key usage. Add them to:
- `00-shared/SCREEN-SPECS.md`
- `API-SCREEN-MAP.csv`
- `INFORMATION-ARCHITECTURE.md`
- `COPY-AND-STATES.md`
- `status-semantics.json`
- `THEMES.md` (chart roles)
- `linux/SCREENS.md`, with the same IDs used in the macOS plan

That keeps the specs repo the source of truth across all six apps.

---

## 6. Technology stack

### 6.1 The Linux app

| Concern | Choice | Why |
|---|---|---|
| Language | **Rust** (stable, 2024 edition) | The spec offers Rust or Python. Rust wins for this app: the memory budget (≤ 250 MB with 10k-row tables), a typed client generated from the 118-operation spec, one core shared with the CLI, a single binary, and no Python runtime inside the Flatpak. |
| UI toolkit | **GTK 4 + libadwaita** via `gtk4-rs` / `libadwaita-rs` (`v1_5` feature minimum, `v1_8` where available) | Spec. Runs on GNOME 46+ and correctly on KDE Plasma. |
| UI definition | **Blueprint** (`.blp`) compiled to `.ui`, using `#[derive(CompositeTemplate)]` widget subclasses | Spec. This is the idiomatic gtk-rs pattern used by Loupe, Fractal and Health. relm4 is skipped because it fights Blueprint templates. |
| Async | `tokio` multi-thread runtime on a worker thread; UI work on the GLib main loop via `glib::spawn_future_local`, bridged with `async-channel` | `reqwest` needs tokio, and the UI must never block |
| API client | Generated from the API repo's `docs/openapi.yaml`, pinned by `info.version`. The spec is **OpenAPI 3.1**, so the plan is to run `progenitor` after an automated 3.1 → 3.0 down-convert, and fall back to `openapi-generator`'s `rust` (reqwest) generator if progenitor can't take the spec. **Spike this in week 1.** A thin hand-written `ApiClient` wraps the generated one. | Spec rule: screens never call HTTP directly, and a spec change becomes a compile error |
| HTTP | One shared `reqwest::Client` with HTTP/2, a keep-alive pool and `rustls` + the OS trust store (`rustls-platform-verifier`, no pinning, per spec). Default headers: `Substratal-Version: 1` and `User-Agent: Substratal-Linux/{ver} ({distro})`. Middleware handles bearer and single-flight refresh, `Idempotency-Key` (UUID v4 created when a form opens), `If-Match`/`ETag`, `RateLimit-*` / `Retry-After` backoff, error-envelope decoding, request coalescing of identical in-flight GETs, and timeouts of 10 s connect / 30 s read. | Efficient over a ~80 ms RTT link: one warm TLS connection, no duplicate polls |
| Secure storage | **`oo7`** (Secret Service, or the Secret portal inside the Flatpak; works with GNOME Keyring and KWallet). It stores only the refresh token and the cache encryption key. | Spec |
| Cache | **SQLite via `rusqlite` with SQLCipher** (`bundled-sqlcipher`) in `$XDG_CACHE_HOME/com.substratalapps.Hub/`, keyed from Secret Service. It caches the catalog, roles, permissions, own profile and settings, the inbox's first page, and the last Overview snapshot (for a sub-second cold start). Admin lists are never cached. | Spec §7; offline read-only |
| Preferences | GSettings schema `com.substratalapps.Hub` (window state, saved log queries, poll intervals) | GNOME idiom; Flatpak-friendly |
| Charts | In-house `ChartView` (`GtkSnapshot`) + `plotters-cairo` | No native GTK chart widget; this keeps dependencies small and themeable |
| Rich views | `GtkSourceView 5` (JSON inspector, settings-schema editor) | Ships in the GNOME runtime |
| Portals | `ashpd` for Background/autostart, OpenURI (Stripe, docs), FileChooser (via `GtkFileDialog`), Notification (via `GNotification`) | Spec §4, "portals first" |
| Session lock | `zbus` listening to logind / `org.gnome.ScreenSaver` | Lock the Console when the desktop locks |
| i18n | `gettext-rs`, with keys mirroring screen IDs; pseudo-locale build | Spec NFR §4 |
| Errors and logging | `thiserror` / `anyhow`; `tracing` to the journal (`tracing-journald`), redacting tokens, emails, IDs and notification content | Spec §11 |
| Build | `meson` + `cargo` (GNOME template layout), `blueprint-compiler`, `glib-compile-resources` | Required by flatpak-builder and Flathub |
| Tests | `cargo nextest` for the core (Capabilities: persona × nav-item matrix; refresh races; idempotency replays); **contract tests against the API's local devserver** (`make up && make run` in Docker Compose, started in CI); screenshot tests per theme with `gtk4` broadway or Xvfb; accessibility checks with `accerciser`/AT-SPI scripted via `dogtail` for the 11 flows | Spec quality gates |
| Lint | `clippy -D warnings`, `rustfmt`, `cargo deny` (licences and advisories), and a custom check rejecting literal colours in CSS and Blueprint | |

### 6.2 Workspace layout

```
substratalapps-linux/
├── Cargo.toml                 workspace
├── crates/
│   ├── substratal-api/        generated client (build.rs runs the generator on vendor/openapi.yaml) + ApiClient middleware
│   ├── substratal-core/       SessionStore, Capabilities, domain services (Directory, Access, Roles, Apps, Keys,
│   │                          Webhooks, Tenants, Orgs, Billing, Audit, Notifications, Ops), cache, flags, telemetry
│   ├── substratal-hub/        GTK app: application.rs, window.rs, router (substratal:// routes), screens/<domain>/,
│   │                          widgets/ (StatusBadge, AccessSwitch, SecretRevealDialog, StepUpDialog, MetricTile,
│   │                          ChartView, TimeRangePicker, LogTable), resources/ (*.blp, theme CSS, icons)
│   └── substratalctl/         CLI on the same core (clap): login, users, access, roles, apps, keys, webhooks,
│                              audit, ops overview|alarms|logs|jobs; `--output json|table`
├── data/                      .desktop, metainfo.xml, GSettings schema, D-Bus service, icons (from brand-assets/)
├── build-aux/flatpak/         com.substratalapps.Hub.json (current stable GNOME runtime, rust-stable SDK extension)
├── vendor/openapi.yaml        pinned copy of the API spec (updated by a script, reviewed in PRs)
└── docs/                      these plans
```

**Why `substratalctl`:** Linux admins live in terminals. The CLI costs little because it reuses `substratal-core`. It gives scriptable admin actions with the same permissions and audit trail, and it runs on a headless EC2 box, in AWS CloudShell, or in a CI job. Ship it as part of the Flatpak (`flatpak run --command=substratalctl …`) and as a static `x86_64`/`aarch64` musl binary.

### 6.3 Distribution and updates (cheap)

| Channel | How | Cost |
|---|---|---|
| **Primary: self-hosted Flatpak repo** | `flatpak-builder` in CI produces an OSTree repo with static deltas, GPG-signed and synced to the existing static-assets **S3 bucket + CloudFront** under `flatpak.substratalapps.com`. Admins install once from a `.flatpakref`; GNOME Software and `flatpak update` handle updates. | Pennies per month (S3 storage plus CloudFront egress at admin-team volume) |
| Beta channel | A second branch (`beta`) in the same repo | $0 |
| Later: Flathub | When the end-user Hub ships (Personal screens complete), publish `com.substratalapps.Hub` on Flathub for discovery. Keep the self-hosted repo for the beta. | $0 |
| Optional | `.deb`/`.rpm` (`cargo-deb`, `cargo-generate-rpm`) as GitHub Release assets for locked-down hosts without Flatpak; a static musl `substratalctl` tarball | $0 |
| CI | **GitHub Actions `ubuntu-24.04`** with `flatpak/flatpak-github-actions` (x86_64, plus aarch64 on `ubuntu-24.04-arm`). It publishes to S3 through the **same GitHub OIDC deploy-role pattern the API repo already uses**, so there are no long-lived AWS keys. | Free minutes for a public repo; otherwise a few dollars a month |

### 6.4 What runs in AWS (and why so little)

The Linux app is a client. Its only AWS footprint is the download hosting above. The ops data comes from **one new Lambda in the API's existing stack** (Plan 2 §3):

```
Linux app / substratalctl ──HTTPS + Substratal-Version: 1 + Bearer (platform session)──▶ API Gateway (existing HTTP API, us-east-1)
                                                       ├─ /*       → api Lambda (existing)
                                                       └─ /ops/*   → ops Lambda (NEW, Go, arm64, own IAM role)
                                                                      ├─ CloudWatch GetMetricData / DescribeAlarms (read)
                                                                      ├─ Logs Insights StartQuery / GetQueryResults (capped)
                                                                      ├─ SQS attributes, DLQ redrive (ops.manage)
                                                                      ├─ Cost Explorer (cached 6 h)
                                                                      ├─ Scheduler + jobs Lambda invoke (ops.manage)
                                                                      └─ Aurora via Data API (usage rollups, config, job runs)
CloudWatch alarm ─▶ SNS (existing) ─▶ ops Lambda ─▶ platform notification (existing inbox) ─▶ Linux app polls ─▶ GNotification
```

| Decision | Reason |
|---|---|
| **No AWS credentials on admin laptops** | One identity system (the API's users, roles and MFA), one permission check (`has_platform_permission` + `ops.*`), and one audit log. Direct AWS SDK calls from the desktop would need IAM Identity Center, a second audit trail and a second offboarding path. |
| **Separate ops Lambda** | It keeps CloudWatch, Cost Explorer and SQS permissions out of the public API Lambda's role. The repo already applies the same reasoning to `mcp` and `stripe-webhook`. |
| **Aggregated endpoint** | The client polls one `/ops/overview` (cached 30 s in the Lambda) instead of ten endpoints. Many open Consoles don't multiply CloudWatch calls. |
| **Avoid** | Amazon Managed Grafana (≈ $9 per editor per month), OpenSearch (a $40+/month floor), per-key custom CloudWatch metrics (≈ $0.30 per metric per month × keys × routes), X-Ray on by default, a VPC or NAT, and a separate dashboard backend |

Estimated added AWS cost: **≈ $2–4/month**, itemized in Plan 2.

---

## 7. Core architecture details

- **Capabilities** (spec §5): computed at sign-in, on window focus and on any `403`, from 6 calls today (`/users/me`, `/users/me/roles`, `/roles?scope=platform`, `/applications?owned=true`, `/tenants`, `/organizations`), run concurrently with `tokio::join!`. It switches to a single `GET /users/me/capabilities` when Plan 2 §7.1 lands. It exposes `can_*` flags, `owns_apps`, `admin_of_orgs`, `plan`, the feature flags, and (new) `can_view_ops` / `can_manage_ops`. Every sidebar row and action binds to a GObject property on a `Capabilities` object, so gating updates live.
- **Feature flags:** a compile-time default, overridden by `GET /platform/config` (Plan 2 §1) when it exists, and turned off by a `404`/`501` from a gated endpoint for the session.
- **Polling scheduler:** one `Poller` in the core owns every periodic task: `unread-count` 60 s, Overview 30 s (visible) / 120 s (unfocused) / off (hidden), alarms 30 s while one is open, and capabilities on focus. On `429` it honours `Retry-After` and doubles background intervals up to 5 min. Polling runs only for `ops.view` holders and pauses on metered connections (`GNetworkMonitor:network-metered`).
- **Errors:** the `{error:{code,message,details}}` envelope maps codes to COPY-AND-STATES strings. `details[].field` maps to `AdwEntryRow` error styling. The raw message and `request_id` appear in a "Details" expander with **Copy request id** and **Search logs** (OPS-04).
- **Conflicts:** `409 version_conflict` refetches and offers **Reload** or **Re-apply my change**.
- **Telemetry:** the spec §12 events plus `screen_view{OPS-xx}`. It's opt-out, carries no PII, and only goes to the API's own endpoint if one is added. Until then it's local `tracing` only.

---

## 8. Build phases

Each phase ends with a Flatpak on the **beta** branch.

| Phase | Scope | API dependency | Size |
|---|---|---|---|
| **A. Foundation** | Workspace, meson/Flatpak, client generator spike, `ApiClient` middleware (version header, refresh, idempotency, ETag, retries), `oo7`, SQLCipher cache, sign-in + TOTP, Capabilities, sidebar + gating, router + `substratal://`, themes (10), system states, idle/desktop lock, CI + S3 Flatpak repo | none | 2–3 wks |
| **B. Administration (spec MVP)** | USR-*, ACX-*, ROL-*, AUD-*, GLB-*; MFA-required gate (client side) | none | 3–4 wks |
| **C. Developer** | APL-01..07, KEY-01..04 + hygiene, WHK-01..04, BIL-04, `substratalctl` v1 (users, access, keys, audit) | none (PR #60 for self-service, #61 for audit export) | 3 wks |
| **D. Ops v1** | OPS-01 with ✅ tiles, cross-webhook failures (looped), polling scheduler + GNotification + Background portal | — | 1–2 wks |
| **E. Ops v2** | OPS-01 full, OPS-02, OPS-03, OPS-04, OPS-05 (read), OPS-06, KEY-05, alarm notifications | Plan 2 R1–R2 | 3 wks after R1 |
| **F. Control** | OPS-05 rules, OPS-07, OPS-08, OPS-09, OPS-10, APL-08, step-up everywhere | Plan 2 R3–R4 | 3 wks |
| **G. Personal Hub** | Home, Inbox, Account, Settings, Orgs, Plan & Billing (Stripe via OpenURI + `substratal://billing/return` poll); completes the 75 screens; Flathub submission | none | 4–5 wks |

B and C start the day A lands. D–F are paced by Plan 2, which runs **in parallel** in the API repo. G can move earlier if end users on Linux need the Hub sooner.

---

## 9. Acceptance criteria (Console-specific, on top of the spec's per-screen DoD)

- [ ] Persona matrix (superadmin, operator, support, billing admin, app owner, plain member): sidebar and every action match §2 and the IA rules. A plain member sees no Operations, Administration or Developer section.
- [ ] No API key, AWS credential or secret persists anywhere: not on disk, in the cache, journal, GSettings or notifications. KEY-03 secrets clear from the clipboard after 60 s.
- [ ] Every destructive admin action is confirmed (and stepped up once available) and produces an audit event with `actor_type: user`.
- [ ] Overview first paint is ≤ 500 ms from cache and ≤ 1.5 s fresh on a warm Lambda. No polling happens while no window is visible, except the opt-in background inbox poll.
- [ ] Every ops screen degrades to "Not available yet" on `404`/`501`.
- [ ] An ops alarm reaches a running Linux Console as a `GNotification` within 60 s of the state change (30 s while one is open).
- [ ] Works inside the Flatpak sandbox on Ubuntu 24.04/26.04 GNOME, Fedora GNOME and KDE Plasma: secrets, OpenURI, file chooser, notifications and background portals.
- [ ] Orca and keyboard-only pass on the 11 spec flows plus OPS-01, OPS-04 and OPS-07. Charts have a table alternative.
- [ ] `substratalctl` covers the same operations with identical permission errors, and exits non-zero on API errors with the request id on stderr.

---

## 10. Decisions to confirm

1. **One app (the Hub, admin-first) rather than a separate Console.** I recommend this (§1), consistent with the macOS plan.
2. **Rust, not Python.** I recommend Rust (§6.1). If the team strongly prefers Python, PyGObject + `httpx` + `openapi-python-client` works, but drop the CLI-sharing benefit or write the CLI in Python too.
3. **Self-hosted Flatpak repo on S3 + CloudFront first, Flathub later.** I recommend this (§6.3).
4. **Ops data through a new `ops` Lambda**, never AWS SDK calls from the desktop (§6.4).
5. **Spec housekeeping in the specs repo:**
   - it still says base URL `…/v1` and `openapi.yaml` draft.3, but the API moved to the `Substratal-Version` header in draft.4 (`/v1/*` now `404`s)
   - OPEN-QUESTIONS #16 says notifications return `501`, but they're built
   - the new OPS screens, chart theme roles and status values (§5.4) need adding
