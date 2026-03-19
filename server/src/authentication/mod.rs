pub mod manager;
pub use manager::AuthenticationManager;

mod htpasswd;
mod ldap;
mod null;
mod traits;
