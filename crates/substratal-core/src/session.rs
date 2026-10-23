//! The platform session (CLIENT-ARCHITECTURE §3).
//!
//! Rules this enforces:
//! 1. **Single-flight refresh.** Concurrent `401`s wait on one refresh; a
//!    refresh token is never used twice (reuse revokes the whole session and
//!    raises a `security.session_reuse_detected` alert).
//! 2. The rotated refresh token is persisted **before** the new access token
//!    is used.
//! 3. Access tokens live in memory only.

use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::models::AuthSession;
use crate::secrets::SecretStore;

pub const REFRESH_TOKEN: &str = "refresh_token";
/// Refresh proactively this long before the access token expires.
pub const REFRESH_MARGIN: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Instant,
    pub session_id: String,
    pub user_id: String,
}

impl Tokens {
    pub fn from_session(s: &AuthSession, now: Instant) -> Self {
        Tokens {
            access_token: s.access_token.clone(),
            refresh_token: s.refresh_token.clone(),
            expires_at: now + Duration::from_secs(s.expires_in),
            session_id: s.session_id.clone(),
            user_id: s.user_id.clone(),
        }
    }

    pub fn needs_refresh(&self, now: Instant) -> bool {
        now + REFRESH_MARGIN >= self.expires_at
    }
}

pub struct SessionStore {
    env_key: String,
    secrets: SecretStore,
    tokens: Mutex<Option<Tokens>>,
    /// Held for the whole refresh round-trip: this is the single flight.
    refresh_gate: Mutex<()>,
}

impl SessionStore {
    pub fn new(env_key: String, secrets: SecretStore) -> Self {
        SessionStore {
            env_key,
            secrets,
            tokens: Mutex::new(None),
            refresh_gate: Mutex::new(()),
        }
    }

    pub fn secrets(&self) -> &SecretStore {
        &self.secrets
    }

    pub async fn current(&self) -> Option<Tokens> {
        self.tokens.lock().await.clone()
    }

    pub async fn is_signed_in(&self) -> bool {
        self.tokens.lock().await.is_some()
    }

    /// Stores a fresh session: refresh token to secure storage first, then
    /// the tokens in memory.
    pub async fn establish(&self, session: &AuthSession) -> crate::Result<Tokens> {
        self.secrets
            .set(&self.env_key, REFRESH_TOKEN, &session.refresh_token)
            .await?;
        let tokens = Tokens::from_session(session, Instant::now());
        *self.tokens.lock().await = Some(tokens.clone());
        Ok(tokens)
    }

    /// The refresh token to restore a session at launch, if one was saved.
    pub async fn saved_refresh_token(&self) -> crate::Result<Option<String>> {
        self.secrets.get(&self.env_key, REFRESH_TOKEN).await
    }

    /// Wipes credentials (sign out, `session_revoked`, failed refresh).
    pub async fn clear(&self) -> crate::Result<()> {
        *self.tokens.lock().await = None;
        self.secrets.delete(&self.env_key, REFRESH_TOKEN).await
    }

    /// Runs `refresh` at most once for concurrent callers. `seen_access` is
    /// the access token the caller was using when it got a `401`: if another
    /// caller already replaced it, the new tokens are returned without a
    /// second refresh.
    pub async fn refresh_single_flight<F, Fut>(
        &self,
        seen_access: Option<&str>,
        refresh: F,
    ) -> crate::Result<Tokens>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = crate::Result<AuthSession>>,
    {
        let _gate = self.refresh_gate.lock().await;
        if let Some(t) = self.current().await {
            let replaced = seen_access.is_some_and(|seen| seen != t.access_token);
            if replaced || !t.needs_refresh(Instant::now()) && seen_access.is_none() {
                return Ok(t);
            }
        }
        let refresh_token = match self.current().await {
            Some(t) => t.refresh_token,
            None => self
                .saved_refresh_token()
                .await?
                .ok_or(crate::Error::SignedOut)?,
        };
        match refresh(refresh_token).await {
            Ok(session) => self.establish(&session).await,
            Err(e) => {
                if matches!(e.status(), Some(401) | Some(403)) {
                    self.clear().await?;
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn session(n: usize) -> AuthSession {
        AuthSession {
            access_token: format!("at_{n}"),
            expires_in: 900,
            refresh_token: format!("rtk_{n}"),
            session_id: "ses_1".into(),
            user_id: "usr_1".into(),
            email_verified: Some(true),
        }
    }

    #[tokio::test]
    async fn concurrent_401s_refresh_once() {
        let store = Arc::new(SessionStore::new("test".into(), SecretStore::memory()));
        store.establish(&session(0)).await.unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let store = store.clone();
            let calls = calls.clone();
            handles.push(tokio::spawn(async move {
                store
                    .refresh_single_flight(Some("at_0"), |rt| {
                        let calls = calls.clone();
                        async move {
                            assert_eq!(rt, "rtk_0", "a refresh token must never be reused");
                            let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
                            tokio::time::sleep(Duration::from_millis(20)).await;
                            Ok(session(n))
                        }
                    })
                    .await
                    .unwrap()
            }));
        }
        for h in handles {
            assert_eq!(h.await.unwrap().access_token, "at_1");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            store.saved_refresh_token().await.unwrap().as_deref(),
            Some("rtk_1")
        );
    }

    #[tokio::test]
    async fn failed_refresh_clears_credentials() {
        let store = SessionStore::new("test".into(), SecretStore::memory());
        store.establish(&session(0)).await.unwrap();
        let err = store
            .refresh_single_flight(Some("at_0"), |_| async {
                Err(crate::Error::from(crate::ApiError::from_body(
                    401,
                    br#"{"error":{"code":"session_revoked","message":""}}"#,
                    None,
                    None,
                )))
            })
            .await
            .unwrap_err();
        assert_eq!(err.code(), Some("session_revoked"));
        assert!(!store.is_signed_in().await);
        assert_eq!(store.saved_refresh_token().await.unwrap(), None);
    }
}
