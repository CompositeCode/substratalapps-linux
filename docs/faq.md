---
layout: default
title: FAQ
nav_order: 8
permalink: /faq/
---

# Frequently asked questions
{: .no_toc }

1. TOC
{:toc}

## Accounts and access

### Why can't I see Administration or Developer?
They only appear if you have permissions or own an app. See [Navigation and permissions]({{ '/reference/navigation/' | relative_url }}).

### An app shows "Managed by Substratal". Why can't I edit it?
Free apps and apps covered by a Standard subscription are managed by the platform. You can turn them off and back on, but not edit, revoke or delete them.

### Someone has two ways into the same app. Which one wins?
Access paths add up: if any path is active, the app is on. The access screen shows the resolved status first, and **Why?** shows every path.

### How do I sign out everywhere?
Your own sessions: **Account › Security**. Someone else's (with `users.manage`): their **Sessions** tab, **Sign out everywhere**.

## Notifications

### I don't get desktop notifications on Linux when the app is closed.
Linux desktops have no push service the API supports, so the Linux app checks for notifications every minute while it's running. Everything still lands in your Inbox.

### How long are notifications kept?
90 days in the Inbox.

## Plans and billing

### What does Standard cost?
$6 per person per month, monthly in US dollars. No annual plan or trial today.

### I paid but still show Free.
The Hub waits up to 30 seconds for Stripe to confirm. If it still says Free, refresh (<kbd>Ctrl+R</kbd>); if that doesn't help, contact support with the time you paid.

## API and integrations

### Why do my old `/v1/...` requests return 404?
The API version moved into the `Substratal-Version` header in October 2026. Drop `/v1` from the path and send `Substratal-Version: 1`. See [Versioning]({{ '/api/versioning/' | relative_url }}).

### Why does the API say `api_key_in_browser`?
API keys only work from servers. In a browser, use the signed-in person's own token.

### Can I call production from the interactive reference?
No, on purpose: production only accepts browser requests from Substratal's own sites. Use `curl`, a generated client, or the local devserver.

### Where do I report a problem?
In the app, open the error's **Details** and copy the request ID, then open an issue on [GitHub](https://github.com/CompositeCode/substratalapps-linux/issues) or contact support.
