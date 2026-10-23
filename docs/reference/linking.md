---
layout: default
title: Linking help from an app
parent: Reference
nav_order: 6
permalink: /reference/linking/
---

# Linking help from an app
{: .no_toc }

For the teams building each Hub app. This help center is the documentation home for all of them: link to it instead of writing help into the app.
{: .fs-6 .fw-300 }

## Per-screen help URLs

Every screen ID has a stable page:

```text
https://compositecode.github.io/substratalapps-linux/screens/<screen-id, lowercase>/
```

For example `USR-01` → `/screens/usr-01/`. Wire each app's Help button (and <kbd>F1</kbd>, <kbd>⌘?</kbd> or the platform equivalent) to the page for the visible screen. The Linux app does this in `substratal_core::config::help_url`.

## Machine-readable index

[`/screens.json`]({{ '/screens.json' | relative_url }}) lists every screen with its title, route, kind, phase, who can use it, the platforms it's on, and its help URL. Use it to build an in-app help search or to check that every screen you ship has a page.

## Keeping it current

Screen pages are generated from the specs package (`linux/SCREENS.md` and `00-shared/SCREEN-SPECS.md`). When the specs change, the Linux repo's **Specs sync** workflow regenerates them, so a new screen gets a help page without anyone writing one. Platform-specific notes (shortcuts, controls) appear on the page where they differ. See [How these docs stay current]({{ '/contributing/' | relative_url }}).

## If the help center moves

If it gets its own domain (for example `help.substratalapps.com`), the paths stay the same. Keep the base URL in one constant in each app.
