pub mod authentication;
pub use authentication::AuthenticationManager;

mod authenticatable;
mod htpasswd_authenticator;
mod ldap_authenticator;
mod null_authenticator;
