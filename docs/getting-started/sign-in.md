---
layout: default
title: Sign in
parent: Getting started
nav_order: 2
permalink: /getting-started/sign-in/
---

# Sign in
{: .no_toc }

1. Open Substratal. If you're not signed in, the **Sign in** window opens ([AUTH-01]({{ '/screens/auth-01/' | relative_url }})).
2. Enter your email and password. **Sign in** turns on when both are filled in.
3. If two-factor authentication is on, enter the 6-digit code from your authenticator app ([AUTH-02]({{ '/screens/auth-02/' | relative_url }})). Lost your phone? Enter one of your recovery codes instead.

## Staying signed in

On Linux the app keeps your session in your desktop's keyring (GNOME Keyring or KWallet, through the Secret Service). Only a refresh token is stored there. Nothing else is saved, and passwords never are. If no keyring is available, the app works but you sign in each time you open it.

Signing out (main menu ☰ › **Sign Out**) ends the session on the server and removes the stored token.

## If sign-in fails

| You see | What it means |
|---|---|
| "Email or password is incorrect." | One of them is wrong. The app never says which. |
| "Too many attempts. Try again in N seconds." | Sign-in is rate-limited. Wait for the countdown. |
| "This account is suspended." | An administrator suspended it. Contact them. |
| "Your session ended. Sign in again." | You signed out elsewhere, or an admin signed you out everywhere. |
| "Update required" | This version of the app uses an API version the service no longer accepts. Update the app. See [API versioning]({{ '/api/versioning/' | relative_url }}). |
