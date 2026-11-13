//! Fixture loading and the small helpers the mock backend is built from.
//! This file only declares and re-exports the modules.

mod convert_address;
mod convert_body;
mod convert_date;
mod convert_mail;
mod fixture_error;
mod fixture_load;
mod fixture_schema;
pub(crate) mod mutate;
pub(crate) mod search;

pub use fixture_error::FixtureError;
pub use fixture_load::Fixture;
