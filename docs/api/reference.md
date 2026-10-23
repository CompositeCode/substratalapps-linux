---
layout: default
title: Interactive reference
parent: API
nav_order: 3
permalink: /api/reference/
---

# Interactive reference
{: .no_toc }

Every operation, parameter, request body and response in the Platform API's `openapi.yaml`, rendered with Swagger UI. Pick the API version you're building against; the reference sends that version in `Substratal-Version` on every **Try it out** request.
{: .fs-6 .fw-300 }

<div class="api-version-picker">
  <label for="api-version"><strong>Substratal-Version</strong></label>
  <select id="api-version">
    {% for v in site.data.api.versions %}<option value="{{ v.version }}" data-spec="{{ v.openapi | relative_url }}"{% if v.version == site.data.api.current %} selected{% endif %}>{{ v.version }} ({{ v.status }}, spec {{ v.spec_version }})</option>
    {% endfor %}
  </select>
  <a id="api-download" class="btn btn-outline" href="{{ site.data.api.versions[0].openapi | relative_url }}">Download openapi.yaml</a>
</div>

**Trying requests:** browsers can't call production from this page, on purpose. The API only accepts browser requests from first-party origins so admin and service credentials never get used from a web page. Use **Local devserver** under Servers (run the API's devserver with `CORS_ALLOWED_ORIGINS=https://compositecode.github.io`), or copy the request as `curl`.
{: .note }

<div id="swagger-ui">
  <noscript>The interactive reference needs JavaScript. The raw document is <a href="{{ site.data.api.versions[0].openapi | relative_url }}">openapi.yaml</a>.</noscript>
</div>

<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.33.3/swagger-ui.css">
<script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.33.3/swagger-ui-bundle.js"></script>
<script src="https://cdn.jsdelivr.net/npm/js-yaml@4.1.0/dist/js-yaml.min.js"></script>
<script>
  (function () {
    var host = document.getElementById("swagger-ui");
    var picker = document.getElementById("api-version");
    var download = document.getElementById("api-download");

    function load() {
      var opt = picker.options[picker.selectedIndex];
      var version = opt.value;
      var specUrl = opt.getAttribute("data-spec");
      download.href = specUrl;
      fetch(specUrl)
        .then(function (r) { if (!r.ok) throw new Error(r.status); return r.text(); })
        .then(function (text) {
          var spec = jsyaml.load(text);
          spec.servers = (spec.servers || []).concat([
            { url: "http://localhost:8080", description: "Local devserver (make run in the API repo)" }
          ]);
          SwaggerUIBundle({
            spec: spec,
            domNode: host,
            deepLinking: true,
            docExpansion: "none",
            defaultModelsExpandDepth: 0,
            displayOperationId: true,
            displayRequestDuration: true,
            filter: true,
            persistAuthorization: true,
            tryItOutEnabled: false,
            // The API version is a header (no /v1 in paths). Send the selected
            // one on every request, including operations that don't declare it.
            requestInterceptor: function (req) {
              req.headers["Substratal-Version"] = version;
              return req;
            }
          });
        })
        .catch(function (err) {
          host.innerHTML = "<p>Couldn't load the specification (" + err.message + "). Download <a href='" + specUrl + "'>openapi.yaml</a>.</p>";
        });
    }
    picker.addEventListener("change", load);
    load();
  })();
</script>
