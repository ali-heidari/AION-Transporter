use rcgen::generate_simple_self_signed;

pub fn default() -> Result<(String,String), Box<dyn std::error::Error>> {
    let subject_alt_names = vec!["localhost".to_string(), "localhost".to_string()];

    let cert_key = generate_simple_self_signed(subject_alt_names).unwrap();

    Ok((cert_key.signing_key.serialize_pem(), cert_key.cert.pem()))
}
