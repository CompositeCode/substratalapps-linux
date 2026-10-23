---
layout: default
title: How these docs stay current
nav_order: 10
permalink: /contributing/
---

# How these docs stay current
{: .no_toc }

This help center is built from the same repository as the Linux app and published by GitHub Actions on every change. Reference pages are generated, not written, so they can't drift from the app.
{: .fs-6 .fw-300 }

1. TOC
{:toc}

## What triggers an update

| Change | Workflow | What happens |
|---|---|---|
| Any push to `main` that touches the app, the docs or the vendored API spec | **Docs** (`.github/workflows/docs.yml`) | Regenerates reference pages from the code, builds the site, snapshots it as a new docs version, deploys to GitHub Pages |
| A pull request | **CI** (`.github/workflows/ci.yml`) | Fails if generated pages are out of date (`cargo xtask gen-docs --check`) or the site doesn't build |
| The Platform API's `openapi.yaml` changes | **API spec sync** (`.github/workflows/api-spec-sync.yml`), daily and on demand | Fetches the spec, diffs it with oasdiff, adds an [API changelog]({{ '/api/changelog/' | relative_url }}) entry, regenerates the API pages, opens a pull request; breaking changes are labelled |
| The client specs package changes | **Specs sync** (`.github/workflows/specs-sync.yml`), daily and on demand | Regenerates the screen registry, themes and screen pages, opens a pull request |

## Generated pages

| Page | Generated from |
|---|---|
| [Screens]({{ '/screens/' | relative_url }}) and every screen page | the specs package, via `data/spec/screens.json` |
| [Keyboard shortcuts]({{ '/reference/shortcuts/' | relative_url }}) | the shortcut table the app binds |
| [Navigation and permissions]({{ '/reference/navigation/' | relative_url }}) | the app's navigation rules, evaluated for each persona |
| [Status meanings]({{ '/reference/statuses/' | relative_url }}) | `status-semantics.json` |
| [substratalctl]({{ '/reference/cli/' | relative_url }}) | the CLI's own argument definitions |
| [Operations]({{ '/api/operations/' | relative_url }}) and [`openapi.yaml`]({{ '/api/v1/openapi.yaml' | relative_url }}) | the vendored API spec, cross-referenced with the screens |

Run `cargo xtask gen-docs` after changing any of those sources, and commit the result.

## Writing pages

Guides, tutorials and the FAQ are Markdown under `docs/` ([just-the-docs](https://just-the-docs.com/)). Every page has an **Edit this page** link. To preview:

```sh
cd docs && bundle install && bundle exec jekyll serve
```

## Versions

Each published build is a new docs version with a frozen copy, listed in [Docs versions]({{ '/versions/' | relative_url }}). Put `[docs:minor]` or `[docs:major]` in a commit's subject line for a bigger bump. Docs versions are separate from the API's `Substratal-Version`.

## Triggering the API sync from the API repo

The sync also runs every 6 hours. To make it immediate, add this step to the API repo's docs workflow after the spec is published. It needs a token with access to this repo in a secret named `LINUX_REPO_DISPATCH_TOKEN`:

```yaml
- name: Tell the Linux Hub the spec changed
  run: |
    curl -fsS -X POST https://api.github.com/repos/CompositeCode/substratalapps-linux/dispatches \
      -H "Authorization: Bearer ${{ secrets.LINUX_REPO_DISPATCH_TOKEN }}" \
      -H "Accept: application/vnd.github+json" \
      -d '{"event_type":"api-spec-updated"}'
```
