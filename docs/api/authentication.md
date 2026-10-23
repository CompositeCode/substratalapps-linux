---
layout: default
title: Authentication
parent: API
nav_order: 2
permalink: /api/authentication/
---

# Authentication
{: .no_toc }

Every request except a few public ones (`/health`, sign-in, sign-up, password reset) sends a bearer token:

```http
Authorization: Bearer <token>
```

1. TOC
{:toc}

## People: platform sessions

The Hub signs people in with email and password:

1. `POST /auth/login` with `{email, password}` returns either a session or, when two-factor is on, an MFA challenge (`mfa_required: true` and an `mfa_token`).
2. `POST /auth/mfa/verify` with the `mfa_token` and a 6-digit `code` (or a `recovery_code`) returns the session.
3. The session has a 15-minute **access token** (a JWT) and a long-lived, single-use **refresh token**.
4. `POST /auth/token/refresh` swaps the refresh token for a new pair. Each refresh token works **once**: reusing one revokes the whole session and raises a security alert. Refresh one request at a time and save the new refresh token before using the new access token.
5. `POST /auth/logout` ends the session.

## Apps: app tokens

A launched app gets a 5-minute **app token** for one Application: from `POST /auth/app-tokens` (embedded launch from the Hub) or through the hosted sign-in with PKCE (`POST /auth/oauth/authorization-codes` and `POST /auth/oauth/token`).

## Servers and agents: API keys

An API key (`satk_live_…` or `satk_test_…`) is a bearer token for your backend or an AI agent. It holds only the permissions it was created with, and an agent key can be restricted from destructive actions. Keys:

- never work from a browser (`400 api_key_in_browser`);
- can't act as a person (endpoints using `me`, or `app-tokens`, return `400 user_token_required`);
- can't manage other keys.

Create and rotate them in the Hub; see [API keys]({{ '/tutorials/api-keys/' | relative_url }}).

## Permissions

What a token can do comes from its holder's roles, or the key's permissions. A `403 forbidden` names the missing permission in `details.required_permission`. See [Navigation and permissions]({{ '/reference/navigation/' | relative_url }}) for the platform permissions.
