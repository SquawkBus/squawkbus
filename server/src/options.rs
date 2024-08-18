// Command line parameters.

use std::path::PathBuf;

use argh::FromArgs;

fn default_endpoint() -> String {
    String::from("0.0.0.0:8080")
}

/// SquawkBus server.
#[derive(FromArgs)]
pub struct Options {
    /// an optional authorizations file.
    #[argh(option, short = 'f')]
    pub authorizations: Option<PathBuf>,

    /// endpoint - defaults to 0.0.0.0:8080
    #[argh(option, short = 'e', default = "default_endpoint()")]
    pub endpoint: String,

    /// use tls
    #[argh(switch, short = 't')]
    pub tls: bool,

    /// cert file
    #[argh(option, short = 'c')]
    pub certfile: Option<PathBuf>,

    /// key file
    #[argh(option, short = 'k')]
    pub keyfile: Option<PathBuf>,
}

impl Options {
    pub fn load() -> Self {
        argh::from_env()
    }
}
