//! Secure storage for the refresh token (CLIENT-ARCHITECTURE §3).
//!
//! Only the platform refresh token is ever persisted, in the Secret Service
//! (GNOME Keyring, KWallet, or the Secret portal inside the Flatpak). Access
//! tokens live in memory. API key secrets, webhook secrets, SCIM tokens and
//! recovery codes are never stored anywhere.

use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(feature = "keyring")]
use crate::error::Error;
use crate::error::Result;

const APP_ATTR: &str = "com.substratalapps.Hub";

pub enum SecretStore {
    /// Process memory only. Used by tests, by `substratalctl --no-keyring`,
    /// and when no Secret Service is reachable (the person signs in each launch).
    Memory(Mutex<HashMap<String, String>>),
    #[cfg(feature = "keyring")]
    Keyring(oo7::Keyring),
}

impl SecretStore {
    pub fn memory() -> Self {
        SecretStore::Memory(Mutex::new(HashMap::new()))
    }

    /// The Secret Service, falling back to memory when it isn't available.
    pub async fn system() -> Self {
        #[cfg(feature = "keyring")]
        {
            match oo7::Keyring::new().await {
                Ok(k) => return SecretStore::Keyring(k),
                Err(e) => eprintln!(
                    "substratal: Secret Service unavailable ({e}); credentials won't be remembered"
                ),
            }
        }
        SecretStore::memory()
    }

    pub fn is_persistent(&self) -> bool {
        !matches!(self, SecretStore::Memory(_))
    }

    /// Stores `value` under `name` for one environment (production, local, …).
    pub async fn set(&self, env: &str, name: &str, value: &str) -> Result<()> {
        match self {
            SecretStore::Memory(m) => {
                m.lock()
                    .unwrap()
                    .insert(format!("{env}/{name}"), value.to_string());
                Ok(())
            }
            #[cfg(feature = "keyring")]
            SecretStore::Keyring(k) => {
                let attrs = attributes(env, name);
                k.create_item(&format!("Substratal ({env}) {name}"), &attrs, value, true)
                    .await
                    .map_err(|e| Error::Secrets(e.to_string()))
            }
        }
    }

    pub async fn get(&self, env: &str, name: &str) -> Result<Option<String>> {
        match self {
            SecretStore::Memory(m) => Ok(m.lock().unwrap().get(&format!("{env}/{name}")).cloned()),
            #[cfg(feature = "keyring")]
            SecretStore::Keyring(k) => {
                let items = k
                    .search_items(&attributes(env, name))
                    .await
                    .map_err(|e| Error::Secrets(e.to_string()))?;
                let Some(item) = items.first() else {
                    return Ok(None);
                };
                let secret = item
                    .secret()
                    .await
                    .map_err(|e| Error::Secrets(e.to_string()))?;
                Ok(Some(String::from_utf8_lossy(&secret).into_owned()))
            }
        }
    }

    pub async fn delete(&self, env: &str, name: &str) -> Result<()> {
        match self {
            SecretStore::Memory(m) => {
                m.lock().unwrap().remove(&format!("{env}/{name}"));
                Ok(())
            }
            #[cfg(feature = "keyring")]
            SecretStore::Keyring(k) => k
                .delete(&attributes(env, name))
                .await
                .map_err(|e| Error::Secrets(e.to_string())),
        }
    }
}

#[cfg(feature = "keyring")]
fn attributes<'a>(env: &'a str, name: &'a str) -> HashMap<&'a str, &'a str> {
    HashMap::from([
        ("application", APP_ATTR),
        ("environment", env),
        ("name", name),
    ])
}

#[allow(dead_code)]
const _: &str = APP_ATTR;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_round_trip_is_per_environment() {
        let s = SecretStore::memory();
        s.set("production", "refresh_token", "rtk_a").await.unwrap();
        s.set("local", "refresh_token", "rtk_b").await.unwrap();
        assert_eq!(
            s.get("production", "refresh_token")
                .await
                .unwrap()
                .as_deref(),
            Some("rtk_a")
        );
        s.delete("production", "refresh_token").await.unwrap();
        assert_eq!(s.get("production", "refresh_token").await.unwrap(), None);
        assert_eq!(
            s.get("local", "refresh_token").await.unwrap().as_deref(),
            Some("rtk_b")
        );
    }
}
