---
layout: default
title: Administering people and access
parent: Guides
nav_order: 2
permalink: /guides/administration/
---

# Administering people and access
{: .no_toc }

For staff with platform permissions, and app owners for their own apps. What you see depends on your permissions; see [Navigation and permissions]({{ '/reference/navigation/' | relative_url }}).
{: .fs-6 .fw-300 }

1. TOC
{:toc}

## Users

[Users]({{ '/screens/usr-01/' | relative_url }}) (needs `users.list`) is a searchable table of every account. Search by name, email or a `usr_` ID; pasting an exact ID jumps straight to that person. Open someone for their [overview]({{ '/screens/usr-03/' | relative_url }}) and tabs:

- **Access** ([ACX-01]({{ '/screens/acx-01/' | relative_url }})): which apps are on for them and why.
- **Roles** ([ROL-04]({{ '/screens/rol-04/' | relative_url }})): their platform and app roles.
- **Sessions** ([USR-04]({{ '/screens/usr-04/' | relative_url }})): where they're signed in. **Sign out everywhere** ends every session at once.
- **Notifications & devices** ([NTF-05]({{ '/screens/ntf-05/' | relative_url }})): what they've been sent and their push devices.

Changing an account (invite, suspend, delete) needs `users.manage`. Suspending ([USR-05]({{ '/screens/usr-05/' | relative_url }})) signs the person out everywhere and turns off their access to every app until you reactivate them.

## Access

[Access]({{ '/screens/acx-05/' | relative_url }}) lists access across the platform (`entitlements.manage`), or, for an app owner, across their own apps. [Grant access]({{ '/screens/acx-02/' | relative_url }}) to give someone an app; the switch on a row turns it off or back on, with **Undo** for 10 seconds (<kbd>Ctrl+Z</kbd>). **Why?** ([ACX-04]({{ '/screens/acx-04/' | relative_url }})) explains how someone has, or doesn't have, an app.

Rows marked **Managed by Substratal** (free apps and Standard subscriptions) can only be turned off and back on.
{: .note }

## Roles

[Roles]({{ '/screens/rol-01/' | relative_url }}) are named sets of permissions. Seed roles (`superadmin`, `support`, `billing_admin` and others) can't be edited. You can only give a role permissions you hold yourself, and only assign roles whose permissions you hold, so nobody can grant themselves more than they have. A role's page lists **People with this role**.

## Audit log

The [Audit log]({{ '/screens/aud-01/' | relative_url }}) (`audit.view`) records every change: who did it (a person, an API key or the system), what changed, and the before and after. Filter by person, app, organization, action or time. Each event shows the request ID, which ties it to the API's logs. Events older than the hot window are archived; a banner says when that applies. Exports of a time range are available behind the `audit_export` flag.

## Send notification

Staff with `notifications.send` can [send a notification]({{ '/screens/ntf-06/' | relative_url }}) to a person on behalf of any app. It lands in their inbox and is pushed to their devices where push is available.
