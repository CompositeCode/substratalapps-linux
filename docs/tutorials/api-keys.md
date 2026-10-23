---
layout: default
title: Create, use and rotate an API key
parent: Tutorials
nav_order: 2
permalink: /tutorials/api-keys/
---

# Create, use and rotate an API key
{: .no_toc }

**You need:** to own the app (for an app-scoped key) or hold `api_keys.manage`.

## Create it

1. **Developer › API Keys › New key** (<kbd>Ctrl+N</kbd>).
2. Name it after what will use it ("Notes backend, production").
3. Pick the scope (your app, or platform), **live** or **test**, and **service** or **agent**. For an AI agent, keep **Restrict destructive actions** on.
4. Tick only the permissions it needs.
5. **Create**. The secret appears once. Copy it into your secret manager (for example AWS Secrets Manager), then confirm you've saved it.

## Use it

From your server, never from a browser:

```sh
curl -s https://api.substratalapps.com/users/usr_123/apps/app_notes/effective-permissions \
  -H "Authorization: Bearer $SUBSTRATAL_KEY" \
  -H "Substratal-Version: 1"
```

## Rotate it

Open the key and select **Rotate**. The new secret is shown once, and the old one stops working. Update your secret store first, then deploy. **Revoke** turns a key off for good.

## Keep keys tidy

The key list shows when each key was last used and when it expires. Revoke keys nobody uses.
