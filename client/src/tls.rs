use std::fs::File;
use std::io;
use std::sync::Arc;

use pki_types::ServerName;
use tokio_rustls::{rustls, TlsConnector};

use crate::options::Options;

pub fn create_tls_connector<'a>(options: &Options) -> (TlsConnector, ServerName<'a>) {
    let mut root_cert_store = rustls::RootCertStore::empty();
    if let Some(cafile) = &options.cafile {
        let mut pem = io::BufReader::new(File::open(cafile).expect("Should open cert file"));
        for cert in rustls_pemfile::certs(&mut pem) {
            root_cert_store.add(cert.unwrap()).unwrap();
        }
    } else {
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth(); // i guess this was previously the default?
    let connector = TlsConnector::from(Arc::new(config));

    let domain = pki_types::ServerName::try_from(options.host.as_str())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dns name"))
        .unwrap()
        .to_owned();

    (connector, domain)
}
