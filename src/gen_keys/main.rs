use rand;
use rsa::{pkcs8::EncodePrivateKey, pkcs8::EncodePublicKey, RsaPrivateKey, RsaPublicKey};

fn main() {
    let mut rng = rand::thread_rng();

    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    println!(
        "{}",
        private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::CRLF)
            .unwrap()
            .as_str()
    );
    println!();
    println!(
        "{}",
        public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::CRLF)
            .unwrap()
            .as_str()
    );

    public_key
        .write_public_key_pem_file("keys/master.pub", rsa::pkcs8::LineEnding::CRLF)
        .unwrap();
    private_key
        .write_pkcs8_pem_file("keys/master.key", rsa::pkcs8::LineEnding::CRLF)
        .unwrap();
}
