---
layout: default
title: Install
parent: Getting started
nav_order: 1
permalink: /getting-started/install/
---

# Install
{: .no_toc }

1. TOC
{:toc}

## Linux

Substratal for Linux is a GNOME app (GTK 4 and libadwaita) that also runs on KDE Plasma and other desktops. It needs GNOME 46 or later, or any desktop with Flatpak.

The app is in development. Flatpak builds come from every change to `main` (the **Flatpak** workflow on GitHub); the Flathub listing comes when the app reaches its first release.
{: .note }

**Build and run from source** (Ubuntu 24.04 or Fedora):

```sh
# Ubuntu: sudo apt install libgtk-4-dev libadwaita-1-dev blueprint-compiler librsvg2-common
# Fedora: sudo dnf install gtk4-devel libadwaita-devel blueprint-compiler
git clone https://github.com/CompositeCode/substratalapps-linux
cd substratalapps-linux
cargo run -p substratal-hub
```

No Linux machine? `scripts/dev.sh` runs the same build in a container, and `scripts/screenshot.sh` renders the app headlessly.

To point the app at the API's local devserver instead of production, start it with `SUBSTRATAL_API=local`. A banner tells you you're not on production.

## macOS, Windows, iOS/iPadOS, Android and Web

These apps are built from the same specification by their own teams. Their install pages will be added here as each app ships.

## The command-line companion

`substratalctl` uses the Linux app's core. It's built with the app (`cargo build -p substratalctl`). See the [CLI tutorial]({{ '/tutorials/cli/' | relative_url }}).
