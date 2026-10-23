---
layout: default
title: Home
nav_order: 1
permalink: /
---

# Substratal Hub help
{: .fs-9 }

How to use the Substratal Hub: launch your apps, read your notifications, manage your plan and organizations, and, if you're staff or an app developer, administer users, access, roles, applications, API keys and webhooks.
{: .fs-6 .fw-300 }

[Get started]({{ '/getting-started/' | relative_url }}){: .btn .btn-primary .mr-2 } [Find a screen]({{ '/screens/' | relative_url }}){: .btn .mr-2 } [API reference]({{ '/api/' | relative_url }}){: .btn }


## One help center for every Hub app

The Hub is the same product on **Linux, macOS, Windows, iOS/iPadOS, Android and the Web**: same screens, same screen IDs, same routes, same words. So this help center serves all of them.

- Every screen has a page at **`/screens/<screen-id>/`**, for example [USR-01 Users]({{ '/screens/usr-01/' | relative_url }}). Each app's **Help** button and <kbd>F1</kbd> open the page for the screen you're on.
- Where an app behaves differently on one platform (keyboard shortcuts, how notifications arrive), the page says so.
- Search (top of the page) covers every guide, tutorial, FAQ and screen.

## Where to start

| I want to… | Go to |
|---|---|
| Install the app and sign in | [Getting started]({{ '/getting-started/' | relative_url }}) |
| Understand a screen I'm looking at | [Screens]({{ '/screens/' | relative_url }}) or press <kbd>F1</kbd> in the app |
| Do a common task step by step | [Tutorials]({{ '/tutorials/' | relative_url }}) |
| Learn an area (people, access, apps, keys) | [Guides]({{ '/guides/' | relative_url }}) |
| Look up a shortcut, status or permission | [Reference]({{ '/reference/' | relative_url }}) |
| Integrate with the Platform API | [API]({{ '/api/' | relative_url }}): versioning, authentication, the OpenAPI reference and every operation |
| Get a quick answer | [FAQ]({{ '/faq/' | relative_url }}) |

## The Platform API version is a header

Every Hub app talks to `https://api.substratalapps.com` and names the API version in a **`Substratal-Version: 1`** request header. There's no `/v1` in any URL any more; `/v1/...` paths return `404`. See [API versioning]({{ '/api/versioning/' | relative_url }}).
{: .important }
