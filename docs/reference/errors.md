---
layout: default
title: Error messages
parent: Reference
nav_order: 5
permalink: /reference/errors/
---

# Error messages
{: .no_toc }

What the Hub's error messages mean and what to do. Every error has a **Details** section with the API's error code and a request ID; quote the request ID when asking for help.
{: .fs-6 .fw-300 }

| Message | Code | What to do |
|---|---|---|
| Email or password is incorrect. | `invalid_credentials` | Check both. Use **Forgot password?** if needed. |
| That code didn't work. | `invalid_mfa_code` | Use the current code from your authenticator, or a recovery code. |
| This account is suspended. | `user_suspended` | Ask an administrator. |
| This test account has expired. | `test_user_expired` | Test accounts last 24 hours unless extended. |
| Your session ended. Sign in again. | `unauthenticated`, `session_revoked` | Sign in again; you return to where you were. |
| You don't have permission to do this. | `forbidden` | Ask an administrator for the permission named in Details. |
| Someone else changed this while you were editing. | `version_conflict` | Choose **Reload** to see their change, or **Re-apply** yours on top. |
| Managed by Substratal — you can only turn it off or back on. | `entitlement_platform_managed` | Free and Standard access can't be edited, only switched. |
| You're doing that too often. | `rate_limited` | Wait for the countdown. |
| Substratal is temporarily unavailable. | `service_unavailable` | Reading retries by itself; try changes again shortly. |
| Something went wrong in the app. | `api_version_required`, `method_not_allowed` | A bug in the app. Update it; report it with the request ID if it continues. |
| Update required | `unsupported_api_version` | Your app's API version has been retired. Update the app. See [Versioning]({{ '/api/versioning/' | relative_url }}). |

The full list of API error codes is in the spec's [Conventions → Errors](https://compositecode.github.io/substratalapps.com/api-reference/conventions/).
