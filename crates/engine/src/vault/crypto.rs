//! Sealing and opening the vault's text. ChaCha20-Poly1305 does the
//! encryption; the cryptography library in use has no 24-byte-nonce form, so every save makes
//! a fresh key for itself from the master key with HKDF-SHA256 (a new random
//! 32-byte salt, the fixed info `vault key v1`) and a fresh random 12-byte
//! nonce: a key is never used for more than one message.

use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};
use ring::hkdf::{HKDF_SHA256, Salt};
use ring::rand::{SecureRandom, SystemRandom};

use super::error::VaultError;
use super::format::{HEADER_LEN, Header, Mode, TAG_LEN};
use super::key_source::MasterKey;

const INFO: &[u8] = b"vault key v1";
const SUBKEY_LEN: usize = 32;

/// `N` random bytes from the system.
pub(super) fn random<const N: usize>() -> Result<[u8; N], VaultError> {
    let mut bytes = [0u8; N];
    SystemRandom::new().fill(&mut bytes).map_err(|_| VaultError::Random)?;
    Ok(bytes)
}

/// The key one save or one open uses, made from the master key and the file's
/// own salt.
fn subkey(master: &MasterKey, salt: &[u8]) -> Result<LessSafeKey, VaultError> {
    let mut bytes = [0u8; SUBKEY_LEN];
    let derived = Salt::new(HKDF_SHA256, salt)
        .extract(master.bytes())
        .expand(&[INFO], HKDF_SHA256)
        .and_then(|okm| okm.fill(&mut bytes));
    let key = derived.map_err(|_| VaultError::Crypto).and_then(|()| {
        UnboundKey::new(&CHACHA20_POLY1305, &bytes).map(LessSafeKey::new).map_err(|_| VaultError::Crypto)
    });
    model::wipe_bytes(&mut bytes);
    key
}

/// The whole file for `plaintext`: header, ciphertext, tag. `plaintext` is
/// overwritten.
pub(super) fn seal(master: &MasterKey, mode: Mode, mut plaintext: Vec<u8>) -> Result<Vec<u8>, VaultError> {
    let header = Header { mode, salt: random()?, nonce: random()? };
    let head = header.encode();
    // Room for the tag from the start, so sealing never moves the text.
    let mut body = Vec::with_capacity(plaintext.len() + TAG_LEN);
    body.extend_from_slice(&plaintext);
    model::wipe_bytes(&mut plaintext);
    let sealed = subkey(master, &header.salt).and_then(|key| {
        key.seal_in_place_append_tag(Nonce::assume_unique_for_key(header.nonce), Aad::from(&head[..]), &mut body)
            .map_err(|_| VaultError::Crypto)
    });
    if let Err(error) = sealed {
        model::wipe_bytes(&mut body);
        return Err(error);
    }
    let mut file = Vec::with_capacity(HEADER_LEN + body.len());
    file.extend_from_slice(&head);
    file.extend_from_slice(&body);
    model::wipe_bytes(&mut body);
    Ok(file)
}

/// The text in `file`, whose header has been read as `header`. A wrong key and
/// a changed byte anywhere are the same failure.
pub(super) fn open(master: &MasterKey, header: &Header, file: &[u8]) -> Result<Vec<u8>, VaultError> {
    let key = subkey(master, &header.salt)?;
    let mut body = file[HEADER_LEN..].to_vec();
    let opened = key
        .open_in_place(Nonce::assume_unique_for_key(header.nonce), Aad::from(&file[..HEADER_LEN]), &mut body)
        .map(|plain| plain.len());
    match opened {
        Ok(length) => {
            body.truncate(length);
            Ok(body)
        }
        Err(_) => {
            model::wipe_bytes(&mut body);
            Err(VaultError::KeyMismatch)
        }
    }
}

#[cfg(test)]
include!("crypto.tests.rs");
