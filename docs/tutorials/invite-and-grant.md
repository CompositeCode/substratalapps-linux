---
layout: default
title: Invite someone and give them an app
parent: Tutorials
nav_order: 1
permalink: /tutorials/invite-and-grant/
---

# Invite someone and give them an app
{: .no_toc }

**You need:** `users.manage` to invite, and `entitlements.manage` (or owning the app) to grant access.

1. Go to **Administration › Users** (or press <kbd>Ctrl+K</kbd> and type `users`).
2. Select **Invite user**, enter their email, and send. They appear with status **Invited** and get an email with a link that's valid for 7 days.
3. Open their row, then the **Access** tab, and select **Grant access**.
4. Choose the app and how they get it (an admin grant, a trial with an end date, …), then **Grant**.
5. The access switch shows **Active**, or **Scheduled** if you set a start date in the future.

To take it away later, turn the switch off. You have 10 seconds to undo (<kbd>Ctrl+Z</kbd>).

## The same with the API

```sh
API=https://api.substratalapps.com
H=(-H "Authorization: Bearer $TOKEN" -H "Substratal-Version: 1" -H "Content-Type: application/json")

# 1. Invite (Idempotency-Key makes a retry safe)
curl -s "${H[@]}" -H "Idempotency-Key: $(uuidgen)" \
  -d '{"email":"dana@example.com"}' "$API/users"

# 2. Grant an app (Idempotency-Key is required here)
curl -s "${H[@]}" -H "Idempotency-Key: $(uuidgen)" \
  -d '{"application_id":"app_notes","source":"admin_grant"}' \
  "$API/users/usr_123/entitlements"
```

Request bodies are in the [API reference]({{ '/api/reference/' | relative_url }}) under `POST /users` and `POST /users/{id}/entitlements`.
