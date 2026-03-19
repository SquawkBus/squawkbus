use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use tokio::sync::Mutex;

use common::MessageStream;
use common::messages::Message;

use crate::authentication::htpasswd::HtpasswdAuthenticator;
use crate::authentication::ldap::LdapAuthenticator;
use crate::authentication::null::NullAuthenticator;
use crate::authentication::traits::Authenticator;
use crate::options::AuthenticationOption;

#[derive(Clone)]
pub struct AuthenticationManager {
    pub authenticator: Arc<Mutex<dyn Authenticator + Send>>,
}

impl AuthenticationManager {
    pub fn new(option: &AuthenticationOption) -> Result<Self> {
        Ok(match option {
            AuthenticationOption::None => AuthenticationManager {
                authenticator: Arc::new(Mutex::new(NullAuthenticator {})),
            },
            AuthenticationOption::Basic(path) => AuthenticationManager {
                authenticator: Arc::new(Mutex::new(HtpasswdAuthenticator::new(&path)?)),
            },
            AuthenticationOption::Ldap(url) => AuthenticationManager {
                authenticator: Arc::new(Mutex::new(LdapAuthenticator::new(url.clone()))),
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

        let auth = self.authenticator.clone();
        let auth = auth.lock().await;

        if method.as_str() != auth.name() {
            let msg = std::format!("invalid method {}", method.as_str());
            return Err(Error::new(ErrorKind::Other, msg));
        }

        auth.authenticate(&credentials).await
    }

    pub async fn reset(&mut self) -> Result<()> {
        let auth = self.authenticator.clone();
        let mut auth = auth.lock().await;
        auth.reset().await
    }
}
