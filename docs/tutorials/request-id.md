---
layout: default
title: Trace a problem with a request ID
parent: Tutorials
nav_order: 4
permalink: /tutorials/request-id/
---

# Trace a problem with a request ID
{: .no_toc }

Every API response carries an `X-Request-Id` (`req_…`). When something fails, it's how support finds exactly what happened.

1. When the Hub shows an error, open **Details**. It shows the error code and the request ID.
2. Copy the request ID and send it to support, or, with `audit.view`, search the [Audit log]({{ '/screens/aud-01/' | relative_url }}). Changes made by that request carry the same ID.
3. Operators can look the request up in the web Hub's [Logs]({{ '/screens/ops-03/' | relative_url }}).

`substratalctl` prints the request ID on every failure:

```text
$ substratalctl users list
substratalctl: 403 forbidden: …
request id: req_01M5MR798VSPN5PFEFT4XHNG7M
```
