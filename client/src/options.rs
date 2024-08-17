use std::path::PathBuf;

use argh::FromArgs;

fn default_host() -> String {
    String::from("127.0.0.1")
}

fn default_port() -> u16 {
    8080
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
}

impl Options {
    pub fn load() -> Self {
        argh::from_env()
    }
}
