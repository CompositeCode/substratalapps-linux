//! Drives `ApiClient` against a stub HTTP server to check the request
//! conventions end to end: the version header, deprecation handling,
//! "update required", and single refresh on 401.

use std::sync::{Arc, Mutex};

use substratal_core::api::Request;
use substratal_core::models::LoginOutcome;
use substratal_core::secrets::SecretStore;
use substratal_core::version::VersionStatus;
use substratal_core::{ApiClient, Environment, Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

#[derive(Clone, Debug)]
struct Seen {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

impl Seen {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

type Handler = Arc<dyn Fn(&Seen) -> (u16, Vec<(&'static str, String)>, String) + Send + Sync>;

async fn stub(handler: Handler) -> (Url, Arc<Mutex<Vec<Seen>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let handler = handler.clone();
            let log = log.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    // Read one request (headers plus any Content-Length body).
                    let n = sock.read(&mut chunk).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
                        continue;
                    };
                    let head = String::from_utf8_lossy(&buf[..end]).to_string();
                    let mut lines = head.lines();
                    let mut first = lines.next().unwrap().split(' ');
                    let method = first.next().unwrap().to_string();
                    let path = first.next().unwrap().to_string();
                    let headers: Vec<(String, String)> = lines
                        .filter_map(|l| {
                            l.split_once(':')
                                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                        })
                        .collect();
                    let len: usize = headers
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                        .map(|(_, v)| v.parse().unwrap())
                        .unwrap_or(0);
                    if buf.len() < end + 4 + len {
                        continue;
                    }
                    buf.drain(..end + 4 + len);
                    let req = Seen {
                        method,
                        path,
                        headers,
                    };
                    log.lock().unwrap().push(req.clone());
                    let (status, extra, body) = handler(&req);
                    let mut resp = format!(
                        "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\n",
                        body.len()
                    );
                    for (k, v) in extra {
                        resp.push_str(&format!("{k}: {v}\r\n"));
                    }
                    resp.push_str("\r\n");
                    resp.push_str(&body);
                    sock.write_all(resp.as_bytes()).await.unwrap();
                }
            });
        }
    });
    (Url::parse(&format!("http://{addr}")).unwrap(), seen)
}

fn client(base: Url) -> ApiClient {
    ApiClient::new(
        Environment::Custom(base),
        SecretStore::memory(),
        "Substratal-Linux/test",
        "en-US",
    )
    .unwrap()
}

const SESSION: &str = r#"{"token_type":"Bearer","access_token":"at_1","expires_in":900,"refresh_token":"rtk_1","session_id":"ses_1","user_id":"usr_1"}"#;

#[tokio::test]
async fn every_request_sends_the_version_header_at_the_host_root() {
    let (base, seen) = stub(Arc::new(|_| {
        (
            200,
            vec![("Substratal-Version", "1".into())],
            r#"{"status":"ok"}"#.into(),
        )
    }))
    .await;
    let api = client(base);
    api.health().await.unwrap();
    let r = seen.lock().unwrap()[0].clone();
    assert_eq!(r.path, "/health", "no /v1 prefix");
    assert_eq!(r.header("substratal-version"), Some("1"));
    assert_eq!(r.header("user-agent"), Some("Substratal-Linux/test"));
}

#[tokio::test]
async fn login_then_authenticated_call_carries_bearer() {
    let (base, seen) = stub(Arc::new(|r| match r.path.as_str() {
        "/auth/login" => (200, vec![], SESSION.into()),
        "/users/me" => (
            200,
            vec![("ETag", "\"v3\"".into())],
            r#"{"id":"usr_1","email":"a@b.c","status":"active"}"#.into(),
        ),
        _ => (
            404,
            vec![],
            r#"{"error":{"code":"not_found","message":""}}"#.into(),
        ),
    }))
    .await;
    let api = client(base);
    assert!(matches!(
        api.login("a@b.c", "pw").await.unwrap(),
        LoginOutcome::Session(_)
    ));
    let me = api.me().await.unwrap();
    assert_eq!(me.data.id, "usr_1");
    assert_eq!(me.etag.as_deref(), Some("\"v3\""));
    let log = seen.lock().unwrap();
    assert_eq!(log[1].header("authorization"), Some("Bearer at_1"));
    assert_eq!(log[0].header("authorization"), None, "login is anonymous");
}

#[tokio::test]
async fn mfa_challenge_is_returned_not_stored() {
    let (base, _) = stub(Arc::new(|_| {
        (
            200,
            vec![],
            r#"{"mfa_required":true,"mfa_token":"mfa_x","methods":["totp"],"expires_in":300}"#
                .into(),
        )
    }))
    .await;
    let api = client(base);
    match api.login("a@b.c", "pw").await.unwrap() {
        LoginOutcome::MfaRequired(c) => assert_eq!(c.mfa_token, "mfa_x"),
        other => panic!("{other:?}"),
    }
    assert!(!api.session().is_signed_in().await);
}

#[tokio::test]
async fn deprecation_and_sunset_are_reported() {
    let (base, _) = stub(Arc::new(|_| {
        (
            200,
            vec![
                ("Deprecation", "@1767225600".into()),
                ("Sunset", "Wed, 01 Jul 2026 00:00:00 GMT".into()),
            ],
            r#"{"status":"ok"}"#.into(),
        )
    }))
    .await;
    let api = client(base);
    let status = api.version_status();
    api.health().await.unwrap();
    match &*status.borrow() {
        VersionStatus::Deprecated {
            deprecated_at,
            sunset,
        } => {
            assert!(deprecated_at.is_some());
            assert_eq!(
                substratal_core::version::fmt_date(sunset.unwrap()),
                "Wed, 01 Jul 2026 00:00:00 GMT"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn unsupported_version_stops_all_traffic() {
    let (base, seen) = stub(Arc::new(|_| {
        (400, vec![], r#"{"error":{"code":"unsupported_api_version","message":"no","details":{"supported":["2"]}}}"#.into())
    }))
    .await;
    let api = client(base);
    match api.health().await {
        Err(Error::UpdateRequired { supported }) => assert_eq!(supported, vec!["2".to_string()]),
        other => panic!("{other:?}"),
    }
    assert!(api.version_status().borrow().is_unsupported());
    assert!(matches!(
        api.health().await,
        Err(Error::UpdateRequired { .. })
    ));
    assert_eq!(
        seen.lock().unwrap().len(),
        1,
        "no request after update-required"
    );
}

#[tokio::test]
async fn expired_access_token_refreshes_once_and_replays() {
    let (base, seen) = stub(Arc::new(|r| {
        match (
            r.method.as_str(),
            r.path.as_str(),
            r.header("authorization"),
        ) {
            ("POST", "/auth/login", _) => (200, vec![], SESSION.into()),
            ("POST", "/auth/token/refresh", _) => (
                200,
                vec![],
                SESSION.replace("at_1", "at_2").replace("rtk_1", "rtk_2"),
            ),
            ("GET", "/users/me", Some("Bearer at_1")) => (
                401,
                vec![],
                r#"{"error":{"code":"unauthenticated","message":""}}"#.into(),
            ),
            ("GET", "/users/me", Some("Bearer at_2")) => {
                (200, vec![], r#"{"id":"usr_1","email":"a@b.c"}"#.into())
            }
            _ => (500, vec![], "{}".into()),
        }
    }))
    .await;
    let api = client(base);
    api.login("a@b.c", "pw").await.unwrap();
    assert_eq!(api.me().await.unwrap().data.id, "usr_1");
    let paths: Vec<_> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.path.clone())
        .collect();
    assert_eq!(
        paths,
        [
            "/auth/login",
            "/users/me",
            "/auth/token/refresh",
            "/users/me"
        ]
    );
    assert_eq!(
        api.session()
            .saved_refresh_token()
            .await
            .unwrap()
            .as_deref(),
        Some("rtk_2")
    );
}

#[tokio::test]
async fn writes_send_idempotency_key_and_if_match() {
    let (base, seen) = stub(Arc::new(|r| match r.path.as_str() {
        "/auth/login" => (200, vec![], SESSION.into()),
        _ => (200, vec![], "{}".into()),
    }))
    .await;
    let api = client(base);
    api.login("a@b.c", "pw").await.unwrap();
    let key = substratal_core::api::new_idempotency_key();
    api.execute_raw(
        Request::post("users")
            .idempotency_key(&key)
            .json(&serde_json::json!({"email":"x@y.z"})),
    )
    .await
    .unwrap();
    api.execute_raw(
        Request::patch("users/usr_1")
            .if_match("\"v3\"")
            .json(&serde_json::json!({})),
    )
    .await
    .unwrap();
    let log = seen.lock().unwrap();
    assert_eq!(log[1].header("idempotency-key"), Some(key.as_str()));
    assert_eq!(log[2].header("if-match"), Some("\"v3\""));
}
