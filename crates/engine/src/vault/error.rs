//! Why a vault could not be opened or saved, as a plain sentence for the
//! footer. No variant ever carries a secret, a key or any text of the vault.

use std::fmt;

/// What went wrong with the vault or its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultError {
    /// The vault file or its folder could not be read; holds the system's words.
    Read(String),
    /// The vault is a link or something other than a plain file.
    NotAFile,
    /// The file is larger than the vault may be.
    TooBig,
    /// Too short, wrong magic, or a header that is not one: not a vault.
    Damaged,
    /// A version of the format this program does not know.
    UnknownVersion,
    /// A mode byte this program does not know.
    UnknownMode,
    /// The vault is there and its key is not.
    KeyMissing,
    /// The key does not open the vault, or the vault was changed.
    KeyMismatch,
    /// The key file cannot be used; holds what is wrong with it.
    BadKeyFile(&'static str),
    /// The decrypted text is not a table of names and texts.
    Contents,
    /// The vault or its key could not be written; holds the system's words.
    Write(String),
    /// The system gave no random numbers.
    Random,
    /// The encryption library refused the key.
    Crypto,
    /// The vault is protected by a passphrase and none has been given in this
    /// run (or the one held is not for this file).
    Locked,
    /// The passphrase given does not open the vault. A passphrase that is
    /// wrong and a ciphertext that was changed fail the same way, so this
    /// cannot tell them apart.
    WrongPassphrase,
    /// The file asks the key derivation for settings out of range (zero, or
    /// more than a vault ever needs), so nothing was computed.
    KdfRefused,
    /// The vault is protected in another way than the key given is for.
    WrongMode,
    /// A new passphrase that is shorter than the rule allows.
    PassphraseTooShort,
    /// The vault is a keychain vault and the system keychain works, but has no
    /// key for it: this computer or user never had it (the folder was moved),
    /// or it was deleted.
    KeychainKeyMissing,
    /// The vault needs the system keychain and there is none here, or it is
    /// locked, refused or did not answer in time. Nothing is lost: the vault
    /// is as it was, and the next start asks again.
    KeychainUnavailable,
    /// The system keychain failed in some other way, in fixed words.
    KeychainOther(&'static str),
    /// A key was stored in the system keychain and what it gave back was not
    /// that key, so it was not used and no vault was written for it.
    KeychainReadBack,
    /// The folder says its vault is protected by a passphrase or the system
    /// keychain, and there is no vault file in it.
    Missing,
    /// The secrets folder is a link or not a folder; holds what is wrong.
    BadFolder(&'static str),
    /// The secrets folder, the vault or the key file can be used by other
    /// users and its permissions could not be tightened; holds which one.
    Permissions(&'static str),
    /// The secrets folder cannot be locked against another copy of the
    /// program changing the vault at the same moment; holds why.
    NoLock(String),
    /// Another copy of the program held the lock on the secrets folder for
    /// longer than a change is allowed to wait.
    LockBusy,
    /// Every name a set-aside vault or key file is kept under is taken.
    TooManySetAside,
    /// A start-over was asked for a vault that opens fine.
    OpensFine,
}

/// What to do when a vault cannot be set up again by a save of its own (the
/// caller adds what comes after: enter it again, or sign in again).
pub const START_OVER_ADVICE: &str = "open Options, go to the Data page and choose \"Start the vault over...\"";

impl VaultError {
    /// Whether the vault is lost: the file or its key is gone or does not
    /// match, not merely out of reach (an unreadable folder, a key file of the
    /// wrong length or a newer format are not lost). A key-file vault that is
    /// lost is set aside by the next save of a secret given now; a vault of
    /// any other kind never is (`SharedVault::update_or_start_over`).
    pub fn can_start_over(&self) -> bool {
        matches!(
            self,
            Self::Damaged | Self::KeyMissing | Self::KeyMismatch | Self::Contents | Self::KeychainKeyMissing | Self::Missing
        )
    }

    /// Whether the explicit, confirmed "Start the vault over" is the way out:
    /// the vault cannot be opened and nothing a save does will change that.
    /// Not when it is only locked (the passphrase has not been given) or the
    /// keychain is only out of reach (both are put right by themselves), not
    /// when the system could not read, write or lock the folder, and not when
    /// the passphrase is only wrong.
    pub fn offers_start_over(&self) -> bool {
        self.can_start_over()
            || matches!(
                self,
                Self::NotAFile
                    | Self::TooBig
                    | Self::UnknownVersion
                    | Self::UnknownMode
                    | Self::BadKeyFile(_)
                    | Self::KdfRefused
                    | Self::KeychainOther(_)
            )
    }

    /// Whether a save of a secret given now deals with this by itself, when
    /// the vault is a key-file vault: it is set aside and a new one started.
    /// (For any other vault, and for every other error, only the explicit
    /// choice does.)
    pub fn saves_start_over(&self) -> bool {
        matches!(self, Self::Damaged | Self::KeyMissing | Self::KeyMismatch | Self::Contents)
    }

    /// What to tell the person when this is why a secret they gave could not
    /// be saved: the way out, when there is one.
    pub fn advice(&self) -> Option<&'static str> {
        self.offers_start_over().then_some(START_OVER_ADVICE)
    }

    /// Whether the error is one of writing the vault (as against opening it),
    /// so a caller that asked for nothing to be written can tell a read that
    /// found nothing usable from a write that failed.
    pub fn while_saving(&self) -> bool {
        matches!(self, Self::Write(_) | Self::Random | Self::Crypto | Self::TooBig)
    }
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const OPEN: &str = "the secrets vault could not be opened";
        match self {
            Self::Read(why) => write!(f, "the secrets vault could not be read: {why}"),
            Self::NotAFile => write!(f, "{OPEN}: the vault is a link or not a plain file"),
            Self::TooBig => write!(f, "{OPEN}: the file is larger than a vault can be"),
            Self::Damaged => write!(f, "{OPEN}: the file is damaged or is not a vault"),
            Self::UnknownVersion => write!(f, "{OPEN}: it was made by a newer version of this program"),
            Self::UnknownMode => write!(f, "{OPEN}: it is protected in a way this program does not know"),
            Self::KeyMissing | Self::KeyMismatch => write!(f, "{OPEN}: the key is missing or does not match"),
            Self::BadKeyFile(why) => write!(f, "{OPEN}: the key file {why}"),
            Self::Contents => write!(f, "{OPEN}: its contents are not in the form this program expects"),
            Self::Write(why) => write!(f, "the secrets vault could not be saved: {why}"),
            Self::Random => write!(f, "the secrets vault could not be saved: the system gave no random numbers"),
            Self::Crypto => write!(f, "the secrets vault could not be used: the encryption library refused the key"),
            Self::Locked => write!(f, "the secrets vault is locked: its passphrase has not been entered"),
            Self::WrongPassphrase => write!(f, "that passphrase does not open the vault"),
            Self::KdfRefused => write!(f, "{OPEN}: the key settings in the file are out of range"),
            Self::WrongMode => write!(f, "{OPEN}: it is protected in another way than the key given"),
            Self::PassphraseTooShort => write!(
                f,
                "that passphrase is too short: it needs at least {} characters",
                model::MIN_PASSPHRASE_CHARS
            ),
            Self::KeychainKeyMissing => write!(
                f,
                "{OPEN}: the system keychain does not have this vault's key (the folder was moved to another \
                 computer or user, or the key was deleted)"
            ),
            Self::KeychainUnavailable => write!(f, "there is no system keychain here (or it is locked)"),
            Self::KeychainOther(why) => write!(f, "the system keychain gave an error: {why}"),
            Self::KeychainReadBack => write!(f, "the system keychain did not give back the key that was stored"),
            Self::Missing => write!(
                f,
                "{OPEN}: the vault file is missing, though this folder was protected by a passphrase or the system keychain"
            ),
            Self::BadFolder(why) => write!(f, "{OPEN}: the secrets folder {why}"),
            Self::Permissions(what) => write!(
                f,
                "{OPEN}: {what} can be used by other users and its permissions could not be tightened"
            ),
            Self::NoLock(why) => write!(
                f,
                "the secrets vault could not be changed: the secrets folder cannot be locked ({why}), so two copies of \
                 the program could overwrite each other's changes"
            ),
            Self::LockBusy => write!(
                f,
                "the secrets vault could not be changed: another copy of the program is changing it; try again in a moment"
            ),
            Self::TooManySetAside => write!(
                f,
                "the secrets vault could not be set aside: the secrets folder already holds 100 set-aside files \
                 (vault.unreadable up to vault.unreadable.99); delete or move some of them first"
            ),
            Self::OpensFine => write!(f, "the secrets vault opens fine, so there is nothing to start over"),
        }
    }
}

impl std::error::Error for VaultError {}

#[cfg(test)]
include!("error.tests.rs");
