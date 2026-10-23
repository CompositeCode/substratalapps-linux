---
layout: default
title: API
nav_order: 7
has_children: true
permalink: /api/
---

# Substratal Platform API
{: .no_toc }

Every Hub app is a client of the Substratal Apps Platform API. Use this section to integrate your own app or tooling with it. The API's full specification, domain model and design decisions live in the [Platform API spec site](https://compositecode.github.io/substratalapps.com/); this section covers what you need day to day.
{: .fs-6 .fw-300 }

| | |
|---|---|
| **Base URL** | `{{ site.data.api.versions[0].base_url }}`. Production; the host root, no version in the path. |
| **API version** | `Substratal-Version: {{ site.data.api.current }}` request header, required. See [Versioning]({{ '/api/versioning/' | relative_url }}). |
| **Specification** | [`openapi.yaml`]({{ '/api/v1/openapi.yaml' | relative_url }}), OpenAPI 3.1, revision **{{ site.data.api.versions[0].spec_version }}**, {{ site.data.api.versions[0].operations }} operations |
| **Authentication** | Bearer tokens: a person's access token or an API key. See [Authentication]({{ '/api/authentication/' | relative_url }}). |
| **Format** | JSON. Errors are `{"error": {"code", "message", "details"}}`. |

## Your first request

```sh
curl -i https://api.substratalapps.com/health
# HTTP/2 200
# {"status":"ok"}

curl -i https://api.substratalapps.com/users/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Substratal-Version: 1"
```

`/health` doesn't need the version header; everything else does.

## Conventions in brief

| Topic | Rule |
|---|---|
| IDs | Opaque strings with a type prefix: `usr_`, `app_`, `org_`, `ent_`, `key_`, `evt_` and more. Use `me` for yourself with a person's token. |
| Pagination | `limit` and `cursor`; read `page.next_cursor` until `page.has_more` is `false`. |
| Creates | Send an `Idempotency-Key` (a UUID) so a retry can't create twice. Some creates require it. |
| Edits | Send `If-Match` with the `ETag` you read; `409 version_conflict` means someone else changed it first. |
| Rate limits | 100 requests a minute per session or key by default; `429` carries `Retry-After`. |
| Request IDs | Every response has `X-Request-Id`. Quote it when asking for help. |

Full detail: [Conventions](https://compositecode.github.io/substratalapps.com/api-reference/conventions/) on the spec site.
