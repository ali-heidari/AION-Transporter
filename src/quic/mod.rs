pub mod client;
pub mod server;

pub fn init(){
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring crypto provider");
}