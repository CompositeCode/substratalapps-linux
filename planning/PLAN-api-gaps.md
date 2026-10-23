# Plan 2: What the API is missing for a management app

> **Update, 2026-10-23.** Merged since this was written: #58 test-user expiry, #60 self-service apps, #61 audit export, #62 SCIM, and #68 `GET /users?role_id=` (§7.2) plus sign out everywhere (`POST /users/{id}/sessions/revoke-all`, part of §4). #69 exposes `Deprecation`/`Sunset` to browsers. The spec is now 1.0.0-draft.9 with 124 operations.
>
> The specs moved the Console to the web Hub with its own Ops gateway (specs OPEN-QUESTIONS #27). The API-side asks for it are tracked as API issues G5–G20 (#42–#57), so §3's `ops` Lambda is one option rather than the plan of record.


**Status:** proposal, 2026-10-21. Once agreed, this belongs in the API repo (`CompositeCode/substratalapps.com`) and follows that repo's "Rules for every change": spec page and changelog first, then `make gen`, contract tests, and expand-only migrations.
**Verified against:** `origin/main` `364ce5b` (2026-10-21, `openapi.yaml` 1.0.0-draft.4, 118 operations), `infra/terraform/prod`, `internal/core`, the open PRs #58/#60/#61/#62, and the live API (`https://api.substratalapps.com`, probed 2026-10-21).
**Companion:** [PLAN-management-app.md](PLAN-management-app.md). The section numbers §1–§10 below are the ones Plan 1 cites. They match the macOS plan's numbering so the API team gets one list from every client team. §10 is new here.

> **Path convention.** The API dropped the `/v1` URL prefix in draft.4; the version is now the `Substratal-Version: 1` header. Every endpoint below is therefore written without `/v1`, as `/ops/...`. The macOS plan's `/v1/ops/...` paths are the same endpoints.

---

## 0. Assessment

**The object-management half of the Console is ready.**
- All 118 operations are implemented and production is up: `GET /health` → `200`, and the version header is enforced (`400 api_version_required` without it).
- The permission model is solid admin-tool material: 13 platform permissions, seed roles, a protected superadmin, no-escalation guards, `restrict_destructive`, ETags, idempotency, 56 audit actions, and the audit hot window.

**Four open PRs close gaps the Console would otherwise list:**

| PR | Adds | Console effect |
|---|---|---|
| #58 test-user expiry | `users.expires_at`, `user.expired` | USR-02 test-user tooling |
| #60 self-service apps | `POST /applications` opens to developers | APL-02 for app owners |
| #61 audit export | `POST/GET /audit-exports` | AUD-01 export (§7.4 below is mostly done) |
| #62 SCIM | `/organizations/{id}/scim-token` | ORG-08 SCIM token, OPS-08 |

**The system-management half isn't there.** The API manages *its customers' data* but exposes nothing about *itself*:

| Area | Today (verified) | Where it actually lives |
|---|---|---|
| Health | `GET /health` → `{"status":"ok"}`, shallow | — |
| Metrics | none | CloudWatch built-ins (API Gateway, Lambda, Aurora, SQS) + 3 `Substratal/Security` metric filters |
| Usage per key/app/route | `api_keys.last_used_at` only | `rate_limit_counters`, which are short-lived. The API Gateway access log format is `{requestId, ip, method, path, status, latency, integration}`, with **no principal, key, app or route template**. |
| Logs | none | CloudWatch Logs: API access log 30 days, api Lambda log group 365 days, others at the module default of 30 |
| Alarms | none | 13 CloudWatch alarms → one SNS topic → one `alarm_email` |
| Configuration | none | env vars, Terraform vars, Go constants (`internal/core/ratelimit.go`) |
| Feature flags | none; the client spec assumes "remote config" | — |
| Jobs / queues | none | 11 EventBridge Scheduler jobs; SQS `webhooks` + DLQ |
| Cost | none | Budgets ($75 warn / $150 hard action), Cost Anomaly Detection |
| Releases | none | GitHub Actions, the S3 artifact bucket, `schema_migrations` |
| Application removal | `GET`/`PATCH` only; `suspended` is the only off switch | — |

**Priority** (Console screens unblocked per unit of work):
- **P0:** §9 `ops.*` permissions, §2 request logging and usage, §3 ops read endpoints, §4 MFA for admins
- **P1:** §5 alerts into notifications, §6 application retirement, §7 small resource gaps
- **P2:** §1 configuration and flags, §8 ops actions, §10 client-efficiency items

---

## §9 (first, because everything depends on it): new platform permissions

| Key | Grants | Seed roles |
|---|---|---|
| `ops.view` | Read every `/ops/*` endpoint and receive ops alerts | superadmin, **new seed role `operator`** |
| `ops.manage` | Ops writes: run jobs, redrive the DLQ, ack/mute alarms, edit alert rules and platform config, maintenance mode | superadmin, operator |

- These are expand-only changes: the catalog enum, a seed-role migration (`0005`+, after #58's `0005_test_user_expiry`, so number it in merge order), and the "Platform permission catalog" table in `domain-model/roles-and-permissions.md`.
- `operator` = `ops.view`, `ops.manage`, `audit.view`, `users.list`.
- Neither permission is app-confinable. A platform API key may hold `ops.view` (for an external status page). `ops.manage` requires a User token, the same rule as `api_keys.manage`.

## §1. Platform configuration and feature flags

**Gap:** nothing an operator might change at runtime can change without a deploy. The client spec's feature flags have no server source.

**Add:**
- Table `platform_settings (key text pk, value jsonb, version int, updated_by, updated_at)` plus `platform_settings_history`.
- `GET /platform/config` with an `ETag`. `ops.view` sees everything. Any authenticated caller gets the public subset (flags and maintenance state), which is what clients fold into Capabilities.
- `PATCH /platform/config` (`ops.manage`, `If-Match` required, step-up for `security.*` and `maintenance.*`). It writes the audit action `platform.config_updated` with before and after. Each key's value is validated by a JSON Schema.
- `GET /platform/config/history?key=`.
- **Initial keys:**
  - `flags.*`: the client spec list, `organizations, org_grants, notifications, push, push_credentials, send_notifications, billing, sso, passkeys, webhooks, app_settings, app_roles, hosted_launch, app_registration_self_service, audit_export, operations`
  - `rate_limits.default_per_principal`, `rate_limits.overrides[{op, limit, window}]`, `rate_limits.per_key_overrides[{key_id, limit}]`. These replace the `principalLimits` map's constants as the source. The api Lambda reads them through a 60 s in-process cache, so no request pays a DB read.
  - `signup.mode` (`open | invite_only | closed`), `signup.allowed_domains`
  - `security.require_mfa_for_platform_roles`, `security.admin_session_idle_minutes`, `security.admin_ip_allowlist`
  - `maintenance.mode` (`off | read_only | full`), `maintenance.message`. While it's on, writes return `503 service_unavailable` with `Retry-After`. `ops.manage` holders bypass it, and `/health` and `/ops/*` stay up.
- **Stays in Terraform/env**, shown read-only through `GET /ops/runtime`: region, CORS origins, email sender, push backend, KMS key ids, the API Gateway stage throttle (`api_throttle_rate`).

## §2. Usage data (per key / app / route / user)

**Gap:** no way to answer "who's calling, how much, with what errors".

**Add (cheap path, no new AWS services):**
1. **One structured log line per request** from the api Lambda, emitted in `internal/core/call.go` after the response. The fields are `{"msg":"request","request_id","op","route","status","latency_ms","principal_type":"user|api_key|anon","principal_id","api_key_id","application_id","tenant_id","test_mode","rate_limited","error_code","api_version"}`. It must never carry tokens, emails or bodies. This is also the row type §3 log search returns.
2. **Extend the API Gateway access log format** with `$context.routeKey`, `$context.integrationLatency`, `$context.identity.userAgent` and `$context.error.message`. This is a Terraform-only change.
3. **Hourly rollup job `usage-rollup`.** Add it to the existing `jobs` Lambda and `local.schedules` map. It runs a Logs Insights query over the last closed hour and upserts into `usage_hourly (hour, application_id, api_key_id, op, status_class, test_mode, requests, errors_4xx, errors_5xx, rate_limited, p50_ms, p95_ms, p99_ms)`. Keep 400 days, then roll into `usage_daily`, which is kept indefinitely.
4. **Endpoints:**
   - `GET /usage?from&to&granularity=hour|day&group_by=application|api_key|op|status_class&application_id&api_key_id&test_mode`. Platform scope requires `ops.view`. **App owners and app-scoped keys read their own app's usage**, which gives developers visibility as well.
   - `GET /api-keys/{id}/usage`, the same shape.
5. **Docs:** a new "Usage" API reference page, and an NFR note that usage is accurate to the hour, with the current hour coming from §3 live metrics.

## §3. Ops read endpoints (new `ops` Lambda)

**Gap:** health, metrics, logs, alarms, queues, jobs, cost and release info are visible only in the AWS console.

**Add** `cmd/ops`, a new Lambda behind the existing API Gateway on `ANY /ops/{proxy+}`.
- It reuses `internal/core`'s auth pipeline, so the version header, `401`/`403`, rate limits and the error envelope behave identically.
- It has **its own IAM role**: read-only CloudWatch, Logs, SQS, Scheduler, Cost Explorer and Budgets, plus `lambda:GetFunction`.
- It's Go on arm64, matching the other functions.

All endpoints require `ops.view`.

| Endpoint | Returns | Source |
|---|---|---|
| `GET /ops/overview` | One call for the dashboard: status, the last hour's requests / 4xx / 5xx / 429 / p95, open alarms, queue depths, ACU, MTD cost, pending reviews and tier changes | aggregates the rows below, cached 30 s |
| `GET /ops/health` | Per component, `{name, status: healthy\|degraded\|down\|unknown, checked_at, detail}`, for Aurora (`select 1` via Data API), KMS (`DescribeKey`), SQS, SES (quota, sandbox flag), Stripe (last webhook received), APNs/FCM/Web Push (last success and failure), and each job's last run | live checks, cached 30 s |
| `GET /ops/metrics?set=traffic\|latency\|lambda\|db\|queues\|security&from&to&period` | Time series | one batched `GetMetricData` per call |
| `GET /ops/logs?filters…&from&to&cursor` | Structured rows (§2.1 request lines, security lines, app errors) | Logs Insights from a **server-built** query with no raw query text from clients, a max 24 h range, a bytes-scanned cap, and cursor pagination |
| `GET /ops/alarms`, `GET /ops/alarms/{name}/history` | State, reason, threshold, history | `DescribeAlarms`, `DescribeAlarmHistory` |
| `GET /ops/queues` | Webhook queue depth, oldest age, DLQ count; outbox pending/failed | SQS attributes + outbox tables |
| `GET /ops/jobs` | Each of the 11 jobs: schedule, last run start/end/result/error, next run | a new `job_runs` table written by `cmd/jobs` around each handler, plus Scheduler `GetSchedule` |
| `GET /ops/deliveries?status=failed&since=` | Failing webhook deliveries across **all** webhooks (today it's only per webhook) | outbox tables |
| `GET /ops/push-stats?application_id&from&to` | Push sent / failed / invalid token per provider per app | push delivery tables |
| `GET /ops/cost` | MTD and forecast vs. the warn and hard budgets, top services | Cost Explorer + Budgets, **cached 6 h in Postgres** (CE is $0.01 per call) |
| `GET /ops/releases` | Deployed commit, deploy time, function versions and aliases, latest migration, `info.version`, previous release | Lambda env `GIT_SHA` / `DEPLOYED_AT` (set by `deploy.yml`), plus `schema_migrations` |
| `GET /ops/runtime` | Read-only, non-secret runtime config | env |

`GET /health` stays as it is, because the smoke test depends on it.

## §4. Admin security

**Gaps (verified):**
1. **MFA isn't enforceable.** The NFR says TOTP is *optional* for every user, including superadmins.
2. **No step-up** for destructive admin operations. A stolen 15-minute access token can suspend users, revoke keys or rotate push credentials.
3. **No admin network restriction.**
4. **Audit covers state changes but not sensitive non-writes.** The 56-value `AuditAction` enum has no event for a data export (`GET /users/{id}/export`), an admin revoking someone else's session (`DELETE /users/{id}/sessions/{sessionId}`), a staff notification send, or a sign-in by a platform-permission holder. Those appear only as security log lines.
5. **No "revoke all sessions" call**, for one user or as an incident control.

**Add:**
- **`security.require_mfa_for_platform_roles`** (§1), default **on**. A user holding any platform permission without TOTP gets `403 mfa_required` on endpoints needing a platform permission. Self-service account endpoints still work, so they can enroll.
- **Step-up:** `POST /auth/step-up {code}` issues a fresh access token with a 5-minute `step_up` claim. Destructive platform operations (the set `restrict_destructive` already classifies in `destructive.go`) require it from User callers acting on someone else, returning `403 step_up_required`. The Console shows a TOTP dialog and retries.
- `security.admin_ip_allowlist` applies to platform-permission use only, never to end users.
- **New audit actions:** `user.data_exported`, `session.revoked` (admin-initiated), `notification.sent` (staff), `auth.admin_login`, `platform.config_updated`, `ops.job_run`, `ops.dlq_redriven`, `ops.alarm_muted`, `application.retired`, `application.deleted`. **New target types:** `session`, `platform`, `job`.
- `DELETE /users/{id}/sessions` (revoke all; `users.manage`, or self).
- `POST /ops/sessions/revoke-all {scope: all|platform_roles|test_mode}` (`ops.manage` + step-up), for incident response.

## §5. Alerts and notifications to admins

**Gap:** alarms email one address. The API already has a complete inbox, with platform notifications to `app_hub`, but ops doesn't use it.

**Add:**
- **Subscribe the `ops` Lambda to the existing SNS alarm topic.** On each state change:
  - write a **platform notification** (`category: ops.alarm`, severity, `action_url: substratal://ops/alerts/{name}`) to every user holding `ops.view`. Apple and Android Hubs get a push, and Linux and Windows pick it up on their 60 s inbox poll.
  - emit the webhook event `ops.alarm.state_changed` to **platform-scoped** webhooks.
- **New webhook events:** `ops.config_changed`, `ops.release_deployed`, `ops.job_failed`. With these, Slack, PagerDuty or a SIEM bridge is just a webhook receiver.
- **Optional built-in senders:** `PUT /ops/alert-routes` (`ops.manage`) for a Slack incoming webhook, PagerDuty Events v2 and email, with secrets KMS-encrypted like webhook secrets today.
- **Missing alarms (Terraform):**
  - p95 latency on hot paths vs. the 150 ms NFR
  - 429 spike
  - push failure rate per provider
  - outbox age over 10 min
  - job failure (from `job_runs`)
  - SES bounce/complaint rate
  - budget thresholds into the same SNS topic (Budgets emails separately today)
  - Lambda errors on the new `ops` function
- **Tunable thresholds:** `PATCH /ops/alarms/{name}` works only for a whitelisted set of user-tunable alarms (`PutMetricAlarm` within bounds). Terraform marks those thresholds `ignore_changes`, so a deploy doesn't undo the Console. Also add `POST /ops/alarms/{name}/mute {until}`.
- **Weekly summary:** a `weekly-report` job emails each `ops.view` holder traffic, errors, cost and top keys (SES, already wired).

## §6. Application lifecycle: removal

**Gap:** `/applications/{id}` supports only `GET` and `PATCH`. `ApplicationReviewStatus` is `approved | pending_review | rejected | suspended`, so an app can be suspended, reversibly, but **never retired or removed**. The brief needs both adding and removing apps.

**Add:**
- **`POST /applications/{id}/retire`** (`applications.manage` + step-up). It sets `review_status → retired`, a new terminal value. In one transaction it:
  - disables every entitlement (`access.revoked`, new reason `application_retired`)
  - revokes the app's API keys
  - deactivates its webhooks
  - deletes its push credentials, deactivating devices with the existing `credential_removed` reason
  - writes `application.retired`
  The rows are kept for audit, and retired apps leave the catalog.
- **`DELETE /applications/{id}`**: a hard delete only when the app was never approved *and* never had an entitlement (a mistaken registration). Otherwise it returns `409 application_in_use`, and the caller should retire it instead.
- **Tenant follow-up:** a Tenant whose last Application retires moves to `status: inactive`.
- **Docs:** add `retired` to the review-lifecycle state diagram in `domain-model/applications.md`. Add `application.retired` to the webhook enum so downstream apps hear about it.
- **Coordinate with PR #60:** self-service registration makes accidental and abandoned registrations more likely, which raises the priority of a clean removal path.

## §7. Small gaps in existing resources

| # | Gap | Add | Used by |
|---|---|---|---|
| 7.1 | Capabilities take 6 calls (client spec §5) | `GET /users/me/capabilities` → platform permissions (union), owned apps and tenants, org-admin orgs, plan, **public flags (§1)**, `mfa_required`, `step_up_valid_until` | every client; faster launch |
| 7.2 | No role-holder lookup (client OQ #5) | `GET /users?role_id=` | ROL-02 |
| 7.3 | No bulk endpoints (client OQ #6) | `POST /batch`: up to 50 sub-requests from an allowlist (entitlement toggle, role assign/remove, suspend), each with its own idempotency key and result, one transaction per item | USR-01, ACX-05 |
| 7.4 | Audit search and stats | Export is covered by **PR #61**. Still missing: `q` free text, and `GET /audit-events/stats?group_by=action\|actor&period=day` | AUD-01, compliance |
| 7.5 | Deliveries only per webhook | covered by `/ops/deliveries` (§3), plus `POST /webhooks/deliveries/redeliver {ids[]}` (§8) | OPS-06 |
| 7.6 | Avatar/icon upload (client OQ #1) | `POST /uploads` → presigned S3 PUT to the static-assets bucket + CloudFront URL | APL-02 icon, ACC-01 |
| 7.7 | Per-key rate-limit visibility | `rate_limit: {limit, remaining_now}` on `GET /api-keys/{id}`, via the existing counter peek | KEY-04 |
| 7.8 | Application owner transfer | Confirm whether `owner_user_id` / `owner_organization_id` are PATCHable (they're filterable today). If not, allow it with `applications.manage` and an audit event. | APL-03 |
| 7.9 | Theme selection storage (client OQ #22) | `light_theme` / `dark_theme` on `Settings`, or the two `app_hub` settings-schema keys | SET-01 on every client |

## §8. Ops actions (`ops.manage`, User token + step-up)

| Endpoint | Does |
|---|---|
| `POST /ops/jobs/{name}/runs` | Invokes the jobs Lambda for that job now; records `job_runs` and `ops.job_run` |
| `POST /ops/queues/{name}/redrive` | SQS `StartMessageMoveTask` from the DLQ back to the source queue |
| `POST /webhooks/deliveries/redeliver {ids[]}` | Bulk redeliver (the single-delivery endpoint exists) |
| `PATCH /platform/config` | Maintenance mode, rate limits, flags (§1) |
| `POST /ops/sessions/revoke-all` | §4 |
| *(Phase 3)* `POST /api-keys/{id}/deliver {secret_arn}` | Rotates the key and writes the new secret straight into a named Secrets Manager secret the ops role may write (tag `substratal:managed-key=true`), so internal keys never touch a clipboard |

**Deliberately not added:** deploys and rollbacks from the Console. They stay in GitHub (`deploy.yml` / `rollback.yml`) behind the OIDC trust and the `production` environment gate. The Console shows release info (§3) and links to the workflows.

## §10. Client-efficiency and Linux/CLI items (new in this plan)

Small items the Linux build surfaced. Each is cheap, and the first two help every client.

| # | Gap | Add | Why |
|---|---|---|---|
| 10.1 | **No conditional GETs.** The spec returns `ETag` but defines no `If-None-Match` / `304` | Honour `If-None-Match` on single-resource GETs, `unread-count`, `/ops/overview` and `/users/me/capabilities` | Desktop clients poll every 30–60 s. A `304` saves the DB read and the response body, and makes polling cheap for both sides. |
| 10.2 | **No push provider for Linux or Windows** (client OQ #2) | Keep polling (it's fine at this scale). Optionally add `GET /users/me/notifications/unread-count?wait=25` as a bounded long-poll, served from the existing table and safely under API Gateway's 30 s limit. | Alarm latency drops from ≤ 60 s to near-real-time without WNS, UnifiedPush or WebSockets |
| 10.3 | **CLI sign-in** for `substratalctl` on headless hosts | An OAuth 2.0 Device Authorization Grant (RFC 8628): `POST /auth/device-codes`, approved in any signed-in Hub or the web dashboard. Until then the CLI uses the embedded email + password + TOTP login. | SSH sessions and CloudShell have no browser, and typing a password into a remote terminal is worse |
| 10.4 | `/health` doesn't echo `Substratal-Version` (other responses do) | Echo it, or document `/health` as exempt in Conventions → Versioning | The client's version check on its first call |
| 10.5 | CORS exposes `ETag, X-Request-Id, RateLimit-*, Substratal-Version`, but **not `Retry-After` or `Audit-Hot-Window-Start`** (client OQ #15) | Add both to `access-control-expose-headers` | Not needed by Linux (native, no CORS), but the Web Hub needs it, and it's found by the same review |
| 10.6 | `User-Agent` is the only way Sessions identify a client | Accept an optional `Substratal-Client: linux/{version}` header and store it on the session | ACC-04 / USR-04 show "Substratal for Linux 1.2" instead of parsing UAs |

---

## Cost of all of the above (Tier 0)

| Item | Monthly |
|---|---|
| `ops` Lambda + API Gateway calls at admin volume | ≈ $0 (free tier) |
| One extra structured log line per request (≈ 300 bytes) | ≈ $0.15 per 1M requests |
| Hourly usage rollup (≈ 720 small Logs Insights queries) | < $0.50 |
| `GetMetricData` behind the 30 s-cached overview | < $0.50 |
| Cost Explorer (cached 6 h) | ≈ $1.20 |
| ≈ 8 new standard alarms | ≈ $0.80 |
| New Postgres tables (`usage_hourly/daily`, `platform_settings`, `job_runs`) | within the existing Aurora floor |
| **Total added** | **≈ $2–4 / month** |

There are no new always-on services: no OpenSearch, Grafana, Redis, VPC or NAT. Conditional GETs (§10.1) partly *offset* the added cost by making client polling nearly free.

---

## Suggested API-repo sequencing

| Release | Contents | Unblocks Console phase |
|---|---|---|
| **R0 (in review now)** | Merge PRs #58, #60, #61, #62 | C (self-service apps, audit export, SCIM, test users) |
| **R1** | §9 permissions + `operator` · §2.1–2.2 request logging · §3 `ops` Lambda: `overview`, `health`, `metrics`, `alarms`, `queues`, `releases` · §7.1 capabilities · §7.2 role filter · §10.1 conditional GETs · §10.4 | D/E (Ops v1) |
| **R2** | §2.3–2.4 usage rollup + `/usage` · §3 `logs`, `jobs` (+ `job_runs`), `deliveries`, `push-stats`, `cost` · §5 alarms → notifications + webhook events | E (Ops v2) |
| **R3** | §4 MFA enforcement, step-up, new audit actions, revoke-all · §6 retire/delete application · §7.3 batch · §7.4 audit search/stats | F (Control) |
| **R4** | §1 platform config + flags + maintenance · §5 alert routes, tunable alarms, weekly report · §8 ops actions · §10.2–10.3, 10.5–10.6 | F (Control), CLI |

---

## Housekeeping found during review

- **Specs repo is behind the API:**
  - It cites `openapi.yaml` 1.0.0-draft.3 and base URL `https://api.substratalapps.com/v1`. The API is draft.4, with **header versioning** (`Substratal-Version: 1`, required), and `/v1/*` paths now return `404`. Every client's base URL and default headers in `CLIENT-ARCHITECTURE.md` §4 need updating.
  - OPEN-QUESTIONS #16 still says notifications return `501`, but they're built (`plan-remaining.md`, 2026-10-20).
- **macOS plan** (`substratalapps-macos/docs/`): its `/v1/ops/*` paths should read `/ops/*` for the same reason.
- **API repo:** `plan-remaining.md` says to check `/health` without `/v1` after the first deploy. That's now confirmed working (`200`, 2026-10-21).
- **Not needed by the Console, but blocking real admin invites:** the dashboard's `verify-email` / `reset-password` / invitation pages (`plan-remaining.md` → First-party web pages). AUTH-07 links land on the web dashboard.
