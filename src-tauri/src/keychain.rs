use thiserror::Error;

const SERVICE: &str = "velumia.langdock";

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("keychain: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("{0}")]
    Message(String),
}

pub fn store_secret(ref_id: &str, secret: &str) -> Result<(), KeychainError> {
    let entry = keyring::Entry::new(SERVICE, ref_id)?;
    entry.set_password(secret)?;
    Ok(())
}

pub fn get_secret(ref_id: &str) -> Result<String, KeychainError> {
    let entry = keyring::Entry::new(SERVICE, ref_id)?;
    Ok(entry.get_password()?)
}

pub fn delete_secret(ref_id: &str) -> Result<(), KeychainError> {
    let entry = keyring::Entry::new(SERVICE, ref_id)?;
    entry.delete_credential()?;
    Ok(())
}

pub fn secret_exists(ref_id: &str) -> bool {
    get_secret(ref_id).is_ok()
}
