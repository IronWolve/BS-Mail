//! The advice every sign-in failure ends with: where, in the program, the
//! user signs in again. One place, so the sentence never drifts apart between
//! the messages that carry it.

/// The way to the account form, as the Options screen has it.
const STEPS: &str = "open Options (F10), choose Accounts, pick the account, press Edit account";

/// For an account whose sign-in ended, was refused or cannot be read.
pub(crate) fn sign_in_again() -> String {
    format!("{STEPS}, and sign in again")
}

/// For an account that has never signed in.
pub(crate) fn sign_in_first() -> String {
    format!("{STEPS}, and sign in")
}

#[cfg(test)]
include!("sign_in_advice.tests.rs");
