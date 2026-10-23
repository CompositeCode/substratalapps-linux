//! The API client layer (CLIENT-ARCHITECTURE §4).
//!
//! Every request goes through [`ApiClient::execute`], which:
//! - sends `Substratal-Version` (the one place the version is set; §4.1),
//!   `User-Agent`, `Accept-Language` and the bearer token;
//! - sends `Idempotency-Key` on creates and `If-Match` on edits when given;
//! - checks the version echo and records `Deprecation` / `Sunset`;
//! - turns `400 unsupported_api_version` into [`Error::UpdateRequired`] and
//!   stops all later traffic;
//! - refreshes once (single-flight) on `401` and replays the request;
//! - retries reads on network errors and `503` with jitter, never writes
//!   without an idempotency key;
//! - decodes the error envelope.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::watch;
use url::Url;

use crate::capabilities::Capabilities;
use crate::config::Environment;
use crate::error::{ApiError, Error, Result};
use crate::models::{
    ApplicationSummary, AuthSession, Health, LoginOutcome, MfaChallenge, Page, Role,
    RoleAssignment, User,
};
use crate::secrets::SecretStore;
use crate::session::SessionStore;
use crate::version::{self, API_VERSION, VERSION_HEADER, VersionStatus};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const READ_RETRIES: u32 = 2;

/// One request, before it's sent.
#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub idempotency_key: Option<String>,
    pub if_match: Option<String>,
    pub authenticated: bool,
}

impl Request {
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Request {
            method,
            path: path.into(),
            query: Vec::new(),
            body: None,
            idempotency_key: None,
            if_match: None,
            authenticated: true,
        }
    }
    pub fn get(path: impl Into<String>) -> Self {
        Self::new(Method::GET, path)
    }
    pub fn post(path: impl Into<String>) -> Self {
        Self::new(Method::POST, path)
    }
    pub fn patch(path: impl Into<String>) -> Self {
        Self::new(Method::PATCH, path)
    }
    pub fn delete(path: impl Into<String>) -> Self {
        Self::new(Method::DELETE, path)
    }
    pub fn query(mut self, key: &str, value: impl ToString) -> Self {
        self.query.push((key.to_string(), value.to_string()));
        self
    }
    pub fn json<T: Serialize>(mut self, body: &T) -> Self {
        self.body = Some(serde_json::to_value(body).expect("serializable body"));
        self
    }
    /// For a create form: pass the key generated when the form opened, so a
    /// retry after a network error can't create twice.
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
    pub fn if_match(mut self, etag: impl Into<String>) -> Self {
        self.if_match = Some(etag.into());
        self
    }
    pub fn anonymous(mut self) -> Self {
        self.authenticated = false;
        self
    }
    fn is_read(&self) -> bool {
        self.method == Method::GET
    }
}

/// New UUID v4 for an `Idempotency-Key` (generate it when a create form opens).
pub fn new_idempotency_key() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// A decoded response with the headers screens care about.
#[derive(Clone, Debug)]
pub struct Response<T> {
    pub data: T,
    pub etag: Option<String>,
    pub request_id: Option<String>,
    /// `Audit-Hot-Window-Start` on audit queries.
    pub audit_hot_window_start: Option<String>,
}

pub struct ApiClient {
    http: reqwest::Client,
    env: Environment,
    base: Url,
    session: Arc<SessionStore>,
    version: watch::Sender<VersionStatus>,
}

impl ApiClient {
    /// `client_name` goes into `User-Agent`, e.g. `Substratal-Linux/0.1.0 (Ubuntu 24.04)`.
    pub fn new(
        env: Environment,
        secrets: SecretStore,
        user_agent: &str,
        accept_language: &str,
    ) -> Result<Self> {
        let mut defaults = HeaderMap::new();
        defaults.insert(
            HeaderName::from_static("substratal-version"),
            HeaderValue::from_static(API_VERSION),
        );
        if let Ok(lang) = HeaderValue::from_str(accept_language) {
            defaults.insert(reqwest::header::ACCEPT_LANGUAGE, lang);
        }
        let http = reqwest::Client::builder()
            .default_headers(defaults)
            .user_agent(user_agent)
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| Error::Network(e.to_string()))?;
        let session = Arc::new(SessionStore::new(env.key(), secrets));
        let (version, _) = watch::channel(VersionStatus::Current);
        Ok(ApiClient {
            http,
            base: env.base_url(),
            env,
            session,
            version,
        })
    }

    pub fn environment(&self) -> &Environment {
        &self.env
    }

    pub fn session(&self) -> &Arc<SessionStore> {
        &self.session
    }

    /// Subscribe to API version status (drives GLB-03 Update available / required).
    pub fn version_status(&self) -> watch::Receiver<VersionStatus> {
        self.version.subscribe()
    }

    fn url(&self, req: &Request) -> Result<Url> {
        let path = req.path.trim_start_matches('/');
        let mut url = self
            .base
            .join(path)
            .map_err(|e| Error::Decode(format!("bad path {path}: {e}")))?;
        if !req.query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(req.query.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        }
        Ok(url)
    }

    /// Sends a request and decodes a JSON body.
    pub async fn execute<T: DeserializeOwned>(&self, req: Request) -> Result<Response<T>> {
        let (status, headers, body) = self.execute_raw(req).await?;
        let bytes = if status == StatusCode::NO_CONTENT || body.is_empty() {
            b"null".as_slice()
        } else {
            &body
        };
        let data = serde_json::from_slice(bytes).map_err(|e| Error::Decode(e.to_string()))?;
        Ok(Response {
            data,
            etag: header(&headers, "etag"),
            request_id: header(&headers, "x-request-id"),
            audit_hot_window_start: header(&headers, "audit-hot-window-start"),
        })
    }

    /// Sends a request; returns status, headers and raw body of a success.
    pub async fn execute_raw(&self, req: Request) -> Result<(StatusCode, HeaderMap, Vec<u8>)> {
        if let VersionStatus::Unsupported { supported } = &*self.version.borrow() {
            return Err(Error::UpdateRequired {
                supported: supported.clone(),
            });
        }
        let mut refreshed = false;
        let mut attempt = 0;
        loop {
            let mut access = None;
            if req.authenticated {
                access = Some(self.access_token().await?);
            }
            match self.send_once(&req, access.as_deref()).await {
                Ok((status, headers, body)) => {
                    self.observe_version(&headers);
                    if status.is_success() {
                        return Ok((status, headers, body));
                    }
                    let err = ApiError::from_body(
                        status.as_u16(),
                        &body,
                        header(&headers, "x-request-id"),
                        retry_after(&headers),
                    );
                    match (status.as_u16(), err.code.as_str()) {
                        (400, "unsupported_api_version") => {
                            let supported = err.supported_versions();
                            self.version.send_replace(VersionStatus::Unsupported {
                                supported: supported.clone(),
                            });
                            return Err(Error::UpdateRequired { supported });
                        }
                        (401, _) if req.authenticated && !refreshed => {
                            refreshed = true;
                            self.refresh(access.as_deref()).await?;
                            continue;
                        }
                        (503, _) if req.is_read() && attempt < READ_RETRIES => {
                            attempt += 1;
                            tokio::time::sleep(
                                err.retry_after
                                    .unwrap_or(backoff(attempt))
                                    .min(Duration::from_secs(10)),
                            )
                            .await;
                            continue;
                        }
                        (409, "idempotency_key_in_flight") if attempt < READ_RETRIES => {
                            attempt += 1;
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                        _ => return Err(Error::from(err)),
                    }
                }
                Err(e) => {
                    let retryable = req.is_read() || req.idempotency_key.is_some();
                    if retryable && attempt < READ_RETRIES {
                        attempt += 1;
                        tokio::time::sleep(backoff(attempt)).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn send_once(
        &self,
        req: &Request,
        access: Option<&str>,
    ) -> Result<(StatusCode, HeaderMap, Vec<u8>)> {
        let mut builder = self.http.request(req.method.clone(), self.url(req)?);
        if let Some(token) = access {
            builder = builder.bearer_auth(token);
        }
        if let Some(key) = &req.idempotency_key {
            builder = builder.header("Idempotency-Key", key);
        }
        if let Some(etag) = &req.if_match {
            builder = builder.header("If-Match", etag);
        }
        if let Some(body) = &req.body {
            builder = builder.json(body);
        }
        let resp = builder
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp
            .bytes()
            .await
            .map_err(|e| Error::Network(e.to_string()))?
            .to_vec();
        Ok((status, headers, body))
    }

    /// Records `Deprecation`/`Sunset`, and logs a mismatched version echo
    /// (a proxy or misconfigured environment; behavior doesn't change).
    fn observe_version(&self, headers: &HeaderMap) {
        if let Some(echo) = header(headers, "substratal-version")
            && echo != API_VERSION
        {
            eprintln!(
                "substratal: API answered with {VERSION_HEADER} {echo}, expected {API_VERSION}"
            );
        }
        let deprecation = header(headers, "deprecation");
        let sunset = header(headers, "sunset");
        if deprecation.is_some() || sunset.is_some() {
            let status = VersionStatus::Deprecated {
                deprecated_at: deprecation.as_deref().and_then(version::parse_deprecation),
                sunset: sunset.as_deref().and_then(version::parse_sunset),
            };
            self.version.send_if_modified(|current| {
                if *current != status {
                    *current = status;
                    true
                } else {
                    false
                }
            });
        }
    }

    async fn access_token(&self) -> Result<String> {
        match self.session.current().await {
            Some(t) if !t.needs_refresh(Instant::now()) => Ok(t.access_token),
            Some(_) => Ok(self.refresh(None).await?.access_token),
            None => Err(Error::SignedOut),
        }
    }

    async fn refresh(&self, seen_access: Option<&str>) -> Result<crate::session::Tokens> {
        self.session
            .refresh_single_flight(seen_access, |refresh_token| async move {
                let req = Request::post("auth/token/refresh")
                    .anonymous()
                    .json(&serde_json::json!({ "refresh_token": refresh_token }));
                // Not `execute`: a refresh must never itself trigger a refresh.
                let (status, headers, body) = self.send_once(&req, None).await?;
                self.observe_version(&headers);
                if status.is_success() {
                    serde_json::from_slice::<AuthSession>(&body)
                        .map_err(|e| Error::Decode(e.to_string()))
                } else {
                    Err(Error::from(ApiError::from_body(
                        status.as_u16(),
                        &body,
                        header(&headers, "x-request-id"),
                        retry_after(&headers),
                    )))
                }
            })
            .await
    }

    // ── Operations used by the skeleton ─────────────────────────────────

    /// `GET /health`: the GLB-03 connectivity probe (exempt from the
    /// version header; sending it anyway is harmless).
    pub async fn health(&self) -> Result<Health> {
        Ok(self.execute(Request::get("health").anonymous()).await?.data)
    }

    /// `POST /auth/login`.
    pub async fn login(&self, email: &str, password: &str) -> Result<LoginOutcome> {
        let req = Request::post("auth/login")
            .anonymous()
            .json(&serde_json::json!({ "email": email, "password": password }));
        let value: Value = self.execute(req).await?.data;
        if value.get("mfa_required").and_then(Value::as_bool) == Some(true) {
            let challenge: MfaChallenge =
                serde_json::from_value(value).map_err(|e| Error::Decode(e.to_string()))?;
            return Ok(LoginOutcome::MfaRequired(challenge));
        }
        let session: AuthSession =
            serde_json::from_value(value).map_err(|e| Error::Decode(e.to_string()))?;
        self.session.establish(&session).await?;
        Ok(LoginOutcome::Session(session))
    }

    /// `POST /auth/mfa/verify` with a 6-digit code or a recovery code.
    pub async fn verify_mfa(&self, mfa_token: &str, code: &str) -> Result<AuthSession> {
        let code = code.trim();
        let body = if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
            serde_json::json!({ "mfa_token": mfa_token, "code": code })
        } else {
            serde_json::json!({ "mfa_token": mfa_token, "recovery_code": code })
        };
        let session: AuthSession = self
            .execute(Request::post("auth/mfa/verify").anonymous().json(&body))
            .await?
            .data;
        self.session.establish(&session).await?;
        Ok(session)
    }

    /// Restores the saved session at launch. `Ok(false)` means sign in.
    pub async fn restore(&self) -> Result<bool> {
        if self.session.saved_refresh_token().await?.is_none() {
            return Ok(false);
        }
        match self.refresh(None).await {
            Ok(_) => Ok(true),
            Err(Error::SignedOut) => Ok(false),
            Err(e) if matches!(e.status(), Some(401) | Some(403)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// `POST /auth/logout`, then wipe credentials whatever the result.
    pub async fn logout(&self) -> Result<()> {
        let result = self
            .execute_raw(Request::post("auth/logout").json(&serde_json::json!({})))
            .await;
        self.session.clear().await?;
        match result {
            Ok(_) | Err(Error::SignedOut) => Ok(()),
            Err(e) if e.status() == Some(401) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// `GET /users/me`.
    pub async fn me(&self) -> Result<Response<User>> {
        self.execute(Request::get("users/me")).await
    }

    /// Computes [`Capabilities`] (CLIENT-ARCHITECTURE §5) from six calls run
    /// concurrently. A `403` on the role catalog means "no admin permissions".
    pub async fn capabilities(&self) -> Result<(User, Capabilities)> {
        let me = self.me().await?.data;
        let (assignments, roles, apps, tenants, orgs) = tokio::join!(
            self.execute::<Page<RoleAssignment>>(
                Request::get("users/me/roles").query("limit", 100)
            ),
            self.execute::<Page<Role>>(
                Request::get("roles")
                    .query("scope", "platform")
                    .query("limit", 100)
            ),
            self.execute::<Page<ApplicationSummary>>(
                Request::get("applications")
                    .query("owned", "true")
                    .query("limit", 100)
            ),
            self.execute::<Page<Value>>(
                Request::get("tenants")
                    .query("owner_user_id", &me.id)
                    .query("limit", 100)
            ),
            self.execute::<Page<Value>>(Request::get("organizations").query("limit", 100)),
        );
        let forbidden_is_empty =
            |e: &Error| matches!(e.status(), Some(403) | Some(404) | Some(501));

        let mut caps = Capabilities::member();
        caps.mfa_enabled = me.mfa_enabled;
        caps.plan = Some(me.plan);

        let assigned: Vec<String> = match assignments {
            Ok(r) => r
                .data
                .data
                .into_iter()
                .filter(|a| a.application_id.is_none())
                .map(|a| a.role_id)
                .collect(),
            Err(e) if forbidden_is_empty(&e) => Vec::new(),
            Err(e) => return Err(e),
        };
        match roles {
            Ok(r) => {
                for role in r.data.data.into_iter().filter(|r| assigned.contains(&r.id)) {
                    caps.permissions.extend(role.permissions);
                }
            }
            Err(e) if forbidden_is_empty(&e) => {}
            Err(e) => return Err(e),
        }
        if let Ok(r) = apps {
            caps.owned_apps = r.data.data.into_iter().map(|a| a.id).collect();
        }
        if let Ok(r) = tenants {
            caps.owned_tenants = r
                .data
                .data
                .iter()
                .filter_map(|t| t.get("id")?.as_str().map(str::to_string))
                .collect();
        }
        if let Ok(r) = orgs {
            caps.admin_of_orgs = r
                .data
                .data
                .iter()
                .filter(|o| o.get("my_role").and_then(Value::as_str) == Some("org_admin"))
                .filter_map(|o| o.get("id")?.as_str().map(str::to_string))
                .collect();
        }
        Ok((me, caps))
    }

    /// `GET /users` (USR-01).
    pub async fn list_users(
        &self,
        q: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<Page<User>> {
        let mut req = Request::get("users").query("limit", limit);
        if let Some(q) = q.filter(|q| !q.is_empty()) {
            req = req.query("q", q);
        }
        if let Some(c) = cursor {
            req = req.query("cursor", c);
        }
        Ok(self.execute(req).await?.data)
    }
}

fn header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let v = header(headers, "retry-after")?;
    if let Ok(secs) = v.trim().parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }
    let when = httpdate::parse_http_date(&v).ok()?;
    when.duration_since(std::time::SystemTime::now()).ok()
}

fn backoff(attempt: u32) -> Duration {
    // 250 ms, 500 ms, … plus up to 50 % jitter.
    let base = 250u64 << attempt.min(5);
    let jitter = (uuid::Uuid::new_v4().as_u128() % (base as u128 / 2 + 1)) as u64;
    Duration::from_millis(base + jitter)
}
