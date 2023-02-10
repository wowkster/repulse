use libaes::Cipher;

use rand::{self, Rng};
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey},
    Oaep, PublicKey, RsaPrivateKey, RsaPublicKey,
};

/**
 * Encrypts the data using AES with a random secret. Then, it encrypts the secret
 *  using RSA and prepends the ciphertext with the encrypted secret (2048-bit / 8 = 256 bytes)
 *
 * Encrypted payload format:
 *
 * |      First 256 Bytes      |     Second 32 Bytes     |      Variable Length      |
 * | < RSA Encrypted Secret >  |   <    AES256 IV    >   | < AES256 Encrypted Data > |
 */
pub fn encrypt(data: &[u8], public_key: &rsa::RsaPublicKey) -> Vec<u8> {
    // Create Random AES Secret
    let aes_secret = random_bytes::<32>();

    // Create arbitrary iv
    let iv = random_bytes::<32>().to_vec();

    // Encrypt data with AES256CBC
    let cipher = Cipher::new_256(&aes_secret);
    let cipher_text = cipher.cbc_encrypt(&iv, &data);

    // Encrypt the AES secret with RSA public key
    let mut rng = rand::thread_rng();
    let padding = Oaep::new::<sha2::Sha256>();
    let enc_secret = public_key
        .encrypt(&mut rng, padding, &aes_secret)
        .expect("failed to encrypt aes_secret");

    // Composite encrypted secret and ciphertext together
    [enc_secret, iv, cipher_text].concat()
}

pub fn decrypt(data: &[u8], private_key: &rsa::RsaPrivateKey) -> Vec<u8> {
    // Extract bytes from data
    let enc_secret = &data[0..256];
    let iv = &data[256..288];
    let cipher_text = &data[288..];

    // Decrypt AES secret using RSA
    let padding = Oaep::new::<sha2::Sha256>();
    let aes_secret = private_key
        .decrypt(padding, &enc_secret)
        .expect("failed to decrypt");

    // Decrypt data
    let cipher = Cipher::new_256(&aes_secret.try_into().unwrap());
    let plain_text = cipher.cbc_decrypt(iv, &cipher_text[..]);

    plain_text
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut rng = rand::thread_rng();

    let mut bytes = vec![0u8; N];
    for x in bytes.iter_mut() {
        *x = rng.gen();
    }

    bytes.try_into().unwrap()
}

pub fn create_rsa_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    const BITS: usize = 2048;

    let mut rng = rand::thread_rng();

    let private_key = RsaPrivateKey::new(&mut rng, BITS).expect("failed to generate a private key");
    let public_key = RsaPublicKey::from(&private_key);

    (private_key, public_key)
}

pub trait RsaExt {
    fn to_string(&self) -> String;
}

impl RsaExt for RsaPrivateKey {
    fn to_string(&self) -> String {
        self.to_pkcs8_pem(rsa::pkcs8::LineEnding::CRLF)
            .unwrap()
            .to_string()
    }
}

impl RsaExt for RsaPublicKey {
    fn to_string(&self) -> String {
        self.to_public_key_pem(rsa::pkcs8::LineEnding::CRLF)
            .unwrap()
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use rsa::{
        pkcs8::{DecodePrivateKey, DecodePublicKey},
        RsaPrivateKey, RsaPublicKey,
    };

    use super::{decrypt, encrypt};

    const DATA: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Vitae turpis massa sed elementum tempus egestas. Fermentum posuere urna nec tincidunt. Amet facilisis magna etiam tempor orci. Sodales ut etiam sit amet. Ut morbi tincidunt augue interdum velit. Vel risus commodo viverra maecenas accumsan lacus vel. Vitae tempus quam pellentesque nec nam aliquam sem et tortor. Et netus et malesuada fames ac turpis egestas. Mauris commodo quis imperdiet massa tincidunt nunc pulvinar. Quam elementum pulvinar etiam non quam lacus suspendisse faucibus. Viverra tellus in hac habitasse. Posuere morbi leo urna molestie at elementum eu facilisis. Sit amet massa vitae tortor condimentum lacinia quis vel eros. In iaculis nunc sed augue. Urna id volutpat lacus laoreet non.";

    #[test]
    fn encrypt_decrypt() {
        const MASTER_PUBLIC_KEY: &'static str = include_str!("../keys/master.pub");
        const MASTER_PRIVATE_KEY: &'static str = include_str!("../keys/master.key");

        let master_private_key = RsaPrivateKey::from_pkcs8_pem(MASTER_PRIVATE_KEY).unwrap();
        let master_public_key = RsaPublicKey::from_public_key_pem(MASTER_PUBLIC_KEY).unwrap();

        let encrypted = encrypt(DATA, &master_public_key);

        let decrypted = decrypt(&encrypted, &master_private_key);

        assert_eq!(
            String::from_utf8_lossy(DATA),
            String::from_utf8_lossy(&decrypted)
        );
    }
}
