---
layout: default
title: Use substratalctl in scripts and CI
parent: Tutorials
nav_order: 5
permalink: /tutorials/cli/
---

# Use substratalctl in scripts and CI
{: .no_toc }

`substratalctl` is the Hub's command-line companion. It shares the Linux app's code, so it behaves the same: same API version header, same sign-in, same errors.
{: .fs-6 .fw-300 }

## Check the API

```sh
$ substratalctl health
ok
$ substratalctl version
substratalctl 0.1.0
API version   Substratal-Version: 1 (header, not in the URL)
API base URL  https://api.substratalapps.com/
```

## Sign in

```sh
substratalctl login --email you@example.com      # prompts for the password, then a 2FA code if needed
substratalctl whoami
```

On a desktop the session goes into your keyring. On a server with no keyring, add `--no-keyring` and sign in within the same command run, or use an API key with `curl` instead.

## Find a screen, or its help

```sh
$ substratalctl route substratal://admin/users/usr_42
USR-03 User — Overview (/admin/users/:userId)
  userId = usr_42
  help: https://compositecode.github.io/substratalapps-linux/screens/usr-03/
$ substratalctl screens --json | jq length
```

## Point at another environment

`--api local` (or `SUBSTRATAL_API=local`) uses the API's devserver on `localhost:8080`. Any URL works too.

Every command and flag is in the [reference]({{ '/reference/cli/' | relative_url }}).
