---
layout: default
title: Register, review and suspend an application
parent: Tutorials
nav_order: 3
permalink: /tutorials/applications/
---

# Register, review and suspend an application
{: .no_toc }

## Register (developers)

1. **Developer › Applications › Register app.** You need a verified email.
2. Give it a name, a slug, a launch URL and an icon. The registration is **Pending review**.
3. While you wait, create a **test** API key and build against test-mode users.

## Review (staff with `applications.manage`)

1. Open [Review]({{ '/screens/apl-06/' | relative_url }}). The queue is oldest first.
2. **Approve** to put the app in the catalog, or **Reject** with notes (required). The developer edits and resubmits. Every decision is in the audit log as `application.review_status_changed`.

## Suspend

If an approved app misbehaves, **Suspend** it from its review panel, with notes saying why. Suspension removes the app from the catalog, blocks new grants and stops new launches. People who already have access keep it, and open sessions aren't cut off; turning off existing people's access is a separate, deliberate step in [Access]({{ '/screens/acx-05/' | relative_url }}). **Reinstate** puts it back.

Retiring or deleting an application isn't available yet: suspension is the off switch today.
{: .note }
