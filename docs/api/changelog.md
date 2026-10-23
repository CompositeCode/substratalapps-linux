---
layout: default
title: API changelog
parent: API
nav_order: 5
permalink: /api/changelog/
---

# API changelog
{: .no_toc }

Changes to the Platform API specification this help center documents, newest first. Entries are added automatically by the **API spec sync** workflow when the API's `openapi.yaml` changes; it compares the old and new specification with [oasdiff](https://github.com/oasdiff/oasdiff) and flags anything breaking. The API team's own record is the [spec changelog](https://compositecode.github.io/substratalapps.com/changelog/).
{: .fs-6 .fw-300 }

<!-- api-changelog:insert-here -->

## 1.0.0-draft.9 (API v1.8.0) — 2026-10-22

Baseline for this help center: 124 operations under `Substratal-Version: 1`.

- **Versioning moved to a header** (draft.4 to draft.5, API v1.7.0 to v1.7.3): `/v1` dropped from every path; `Substratal-Version` required; `Deprecation` and `Sunset` on deprecated versions; `405 method_not_allowed` for a wrong method on a known path.
- **SCIM 2.0 provisioning** per organization: `GET`/`POST`/`DELETE /organizations/{id}/scim-token`.
- **Test-mode user expiry**: `expires_at` on test users, `403 test_user_expired`.
- **Self-service application registration**: any person with a verified email can `POST /applications`.
- **Audit exports**: `POST /audit-exports`, `GET /audit-exports/{id}`.
- **Sign out everywhere**: `POST /users/{id}/sessions/revoke-all`; `GET /users?role_id=` finds a role's holders.
- `Deprecation` and `Sunset` exposed to browsers through CORS.
