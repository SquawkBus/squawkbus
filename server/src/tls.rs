use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::{Path, PathBuf};

use pki_types::{CertificateDer, PrivateKeyDer};

use rustls_pemfile::{certs, private_key};

use tokio_rustls::rustls;

pub fn rustls_server_config(
    certfile: &PathBuf,
    keyfile: &PathBuf,
) -> io::Result<rustls::ServerConfig> {
    let certs = load_certs(certfile)?;
    let key = load_key(keyfile)?;

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
}

fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))?)
}
