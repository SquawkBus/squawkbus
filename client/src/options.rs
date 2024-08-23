use std::path::PathBuf;

use argh::FromArgs;

fn default_host() -> String {
    String::from("127.0.0.1")
}

fn default_port() -> u16 {
    8080
}

fn default_authentication_mode() -> String {
    String::from("none")
}

/// SquawkBus client test program.
#[derive(FromArgs)]
pub struct Options {
    /// host
    #[argh(option, short = 'h', default = "default_host()")]
    pub host: String,

    /// port
    #[argh(option, short = 'p', default = "default_port()")]
    pub port: u16,

    /// use tls
    #[argh(switch, short = 't')]
    pub tls: bool,

    /// ca file
    #[argh(option, short = 'c')]
    pub cafile: Option<PathBuf>,

    /// authentication mode
    #[argh(option, short = 'm', default = "default_authentication_mode()")]
    pub authentication_mode: String,

    /// user name
    #[argh(option, short = 'U')]
    pub username: Option<String>,

    /// password
    #[argh(option, short = 'P')]
    pub password: Option<String>,
}

impl Options {
    pub fn load() -> Self {
        argh::from_env()
    }
}
