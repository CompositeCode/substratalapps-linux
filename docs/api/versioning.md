---
layout: default
title: Versioning
parent: API
nav_order: 1
permalink: /api/versioning/
---

# API versioning
{: .no_toc }

The API version is a **request header**, not part of the URL.
{: .fs-6 .fw-300 }

1. TOC
{:toc}

## The header

```http
GET /users/me HTTP/1.1
Host: api.substratalapps.com
Authorization: Bearer eyJ…
Substratal-Version: 1
```

- **Required.** Without it: `400 api_version_required`. With a version the server doesn't serve: `400 unsupported_api_version`. Both list the versions it does serve in `details.supported`. There's no default, so you're never moved to a new version without asking.
- **Echoed.** Every response to a versioned request carries `Substratal-Version` with the version that served it.
- **Whole numbers.** `1` is the only version today.
- **Exempt:** `GET /health`, `POST /auth/oauth/token`, `/.well-known/*` and SCIM (`/scim/v2`). Sending the header to them anyway is harmless.

## No `/v1` in the URL

Until API v1.7.0 (`openapi.yaml` 1.0.0-draft.4, October 2026) paths started with `/v1`. They don't any more:

| Before | Now |
|---|---|
| `POST https://api.substratalapps.com/v1/users` | `POST https://api.substratalapps.com/users` with `Substratal-Version: 1` |

Old `/v1/...` paths return `404 not_found`; there's no redirect. Because the version isn't in the path, a resource's URL stays the same in every future version: links, caches and stored URLs never change.
{: .important }

## What changes within a version, and what doesn't

**Additive changes ship within a version**, with no header change: new endpoints, new optional fields, new enum values. Write clients that ignore fields they don't know and treat unknown enum values as "other".

**Breaking changes ship as the next version number.** The previous version keeps working for **at least 6 months**.

## Deprecation and sunset

When the version you send is deprecated, every response carries two extra headers:

```http
Deprecation: @1798761600
Sunset: Wed, 01 Jul 2027 00:00:00 GMT
```

- `Deprecation` ([RFC 9745](https://www.rfc-editor.org/rfc/rfc9745)): when the version was deprecated.
- `Sunset` ([RFC 8594](https://www.rfc-editor.org/rfc/rfc8594)): when it stops working. After that, requests naming it get `400 unsupported_api_version`.

The Hub apps show **Update available**, with the sunset date, when they see these headers, and **Update required** once the version is retired. Your own clients should log and alert on them.

## Versions

| `Substratal-Version` | Status | Specification | Operations |
|---|---|---|---|
{% for v in site.data.api.versions %}| `{{ v.version }}` | {{ v.status }} | [openapi.yaml]({{ v.openapi | relative_url }}) ({{ v.spec_version }}) | {{ v.operations }} |
{% endfor %}

Revisions of the specification within a version (`1.0.0-draft.N`) are listed in the [API changelog]({{ '/api/changelog/' | relative_url }}). Older revisions of this help center, and the API reference as it was then, are in [Docs versions]({{ '/versions/' | relative_url }}).

## Not the webhook version

A webhook subscription's `api_version` is a **date** (such as `2026-10-05`) that fixes the shape of the payloads it receives. It's versioned separately from `Substratal-Version`, and the Hub labels it **Payload version**.
