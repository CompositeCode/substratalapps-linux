---
layout: default
title: Applications, keys and webhooks
parent: Guides
nav_order: 3
permalink: /guides/developer/
---

# Applications, keys and webhooks
{: .no_toc }

For app developers (anyone who owns an Application) and staff with developer permissions.
{: .fs-6 .fw-300 }

1. TOC
{:toc}

## Applications

An **Application** is an app in the Substratal catalog. [Applications]({{ '/screens/apl-01/' | relative_url }}) lists the ones you own, or all of them with `applications.manage`.

- **Register**: any person with a verified email can [register an application]({{ '/screens/apl-02/' | relative_url }}) (behind the `app_registration_self_service` flag). It starts as **Pending review**.
- **Review**: staff with `applications.manage` approve, reject (with notes) or suspend from [Review]({{ '/screens/apl-06/' | relative_url }}). Suspending is the off switch for a misbehaving app.
- **Configure**: the [overview]({{ '/screens/apl-03/' | relative_url }}) has the launch URL, icon, visibility and access tier; [settings schema]({{ '/screens/apl-04/' | relative_url }}) defines per-user settings; [permissions & roles]({{ '/screens/apl-05/' | relative_url }}) define what people can do inside the app; [push credentials]({{ '/screens/apl-07/' | relative_url }}) hold its APNs, FCM and Web Push keys.

Push key files (`.p8`, FCM JSON) are read once from the file you pick and sent to the API. The Hub never keeps a copy.
{: .note }

## API keys

[API keys]({{ '/screens/key-01/' | relative_url }}) let your backend, or an AI agent, call the API without a person signing in.

- A key is **platform** or **app** scoped, **live** or **test**, and intended for a **service** or an **agent**. Agent keys get `restrict_destructive` on by default, so they can't delete or revoke things.
- You can only give a key permissions you hold.
- The secret (`satk_live_…` or `satk_test_…`) is shown **once**, when the key is created or rotated ([KEY-03]({{ '/screens/key-03/' | relative_url }})). Copy it straight into your secret store. The Hub clears it from the clipboard after 60 seconds and never saves it.
- **Rotate** replaces the secret; **Revoke** turns the key off for good.
- Keys are for servers. The API refuses them from a browser (`400 api_key_in_browser`).

See the [API key tutorial]({{ '/tutorials/api-keys/' | relative_url }}).

## Webhooks

[Webhooks]({{ '/screens/whk-01/' | relative_url }}) send events (access granted or revoked, users suspended, devices registered and more) to your HTTPS endpoint, signed with a secret. [Deliveries]({{ '/screens/whk-03/' | relative_url }}) shows every attempt; you can redeliver one.

A webhook's **payload version** (a date such as `2026-10-05`) fixes the shape of the events it receives. It's separate from the API version header.
{: .important }

## Tenant

Your [Tenant]({{ '/screens/bil-04/' | relative_url }}) is where your apps' data lives. Every Tenant starts on shared infrastructure; support handles moves to dedicated infrastructure through tier-change requests.
