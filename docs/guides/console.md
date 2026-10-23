---
layout: default
title: Operations console (web)
parent: Guides
nav_order: 4
permalink: /guides/console/
---

# Operations console (web)
{: .no_toc }

The web Hub has an **Operations** section for the people who run Substratal: is production healthy, what's slow or failing, what it costs, and what's deployed. It's web-only for now; the Linux, macOS, Windows and mobile apps don't include it (specs OPEN-QUESTIONS #27).
{: .fs-6 .fw-300 }

| Screen | What it answers |
|---|---|
| [OPS-01 Overview]({{ '/screens/ops-01/' | relative_url }}) | Is production healthy right now? |
| [OPS-02 Traffic & latency]({{ '/screens/ops-02/' | relative_url }}) | Trends and service-level objectives |
| [OPS-03 Logs]({{ '/screens/ops-03/' | relative_url }}) | What happened to one request, person or key |
| [SEC-01 Security events]({{ '/screens/sec-01/' | relative_url }}) | Failed sign-ins, lockouts, blocked actions |
| [OPS-04 Alarms]({{ '/screens/ops-04/' | relative_url }}) | What's firing and who's told |
| [OPS-05 Jobs & delivery]({{ '/screens/ops-05/' | relative_url }}) | Scheduled jobs and webhook/push delivery |
| [OPS-06 Releases]({{ '/screens/ops-06/' | relative_url }}) | What's deployed |
| [OPS-07 Cost]({{ '/screens/ops-07/' | relative_url }}) | Spend against budgets |
| [CFG-01 Configuration]({{ '/screens/cfg-01/' | relative_url }}) / [CFG-02 Feature flags]({{ '/screens/cfg-02/' | relative_url }}) | What production runs with |
| [RPT-01 Reports]({{ '/screens/rpt-01/' | relative_url }}) | Growth, access and usage over time |
| [INT-01 Integrations]({{ '/screens/int-01/' | relative_url }}) | Every way to connect other tools |

All of it is read-only in its first version and refreshes at most once a minute.
