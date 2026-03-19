use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use tokio::sync::Mutex;

use common::MessageStream;
use common::messages::Message;

use crate::authentication::authenticatable::Authenticatable;
use crate::authentication::htpasswd_authenticator::HtpasswdAuthenticationManager;
use crate::authentication::ldap_authenticator::LdapAuthenticationManager;
use crate::authentication::null_authenticator::NullAuthenticationManager;
use crate::options::AuthenticationOption;

#[derive(Clone)]
pub struct AuthenticationManager {
    pub auth: Arc<Mutex<dyn Authenticatable + Send>>,
}

impl AuthenticationManager {
    pub fn new(option: &AuthenticationOption) -> Result<Self> {
        Ok(match option {
            AuthenticationOption::None => AuthenticationManager {
                auth: Arc::new(Mutex::new(NullAuthenticationManager {})),
            },
            AuthenticationOption::Basic(path) => AuthenticationManager {
                auth: Arc::new(Mutex::new(HtpasswdAuthenticationManager::new(&path)?)),
            },
            AuthenticationOption::Ldap(url) => AuthenticationManager {
                auth: Arc::new(Mutex::new(LdapAuthenticationManager::new(url.clone()))),
            },
        })
    }

    pub async fn authenticate(&self, stream: &mut impl MessageStream) -> Result<String> {
        let message = stream.read().await?;
        let Message::AuthenticationRequest {
            method,
            credentials,
        } = message
        else {
            return Err(Error::new(
                ErrorKind::Other,
                "expected authentication request",
            ));
        };

        let auth = self.auth.clone();
        let auth = auth.lock().await;

        if method.as_str() != auth.name() {
            let msg = std::format!("invalid method {}", method.as_str());
            return Err(Error::new(ErrorKind::Other, msg));
        }

        auth.authenticate(&credentials).await
    }

    pub async fn reset(&mut self) -> Result<()> {
        let auth = self.auth.clone();
        let mut auth = auth.lock().await;
        auth.reset().await
    }
}
