---
layout: default
title: App changelog
nav_order: 9
permalink: /changelog/
---

# Substratal for Linux changelog
{: .no_toc }

## Unreleased

- **First skeleton.** The GTK 4 + libadwaita app with the full navigation tree, gated by your permissions; all 75 Hub screens reachable by route and `substratal://` link, each laid out from the spec; live sign-in with two-factor; the Users table; Settings › Appearance with all 10 themes; the command palette (<kbd>Ctrl+K</kbd>), keyboard shortcuts, About, and help on <kbd>F1</kbd>.
- Talks to `https://api.substratalapps.com` with `Substratal-Version: 1` on every request; handles **Update available** (`Deprecation`/`Sunset`) and **Update required** (`unsupported_api_version`).
- `substratalctl` command-line companion: `health`, `version`, `login`, `logout`, `whoami`, `users list`, `screens`, `route`, `help-url`.
