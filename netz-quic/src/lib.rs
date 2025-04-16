mod conn;

pub use conn::*;

pub static CA_CERT_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/certs/ca-cert.pem");

pub static SERVER_CERT_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/certs/server-cert.pem");
pub static SERVER_KEY_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/certs/server-key.pem");

pub static CLIENT_CERT_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/certs/client-cert.pem");
pub static CLIENT_KEY_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/certs/client-key.pem");
