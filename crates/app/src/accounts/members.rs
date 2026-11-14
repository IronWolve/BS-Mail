//! Turning what was found into the members of the combined backend.

use engine::{Member, RealBackend};
use model::AccountId;

use super::scan::Found;

/// One member per account, in order: a running account for a usable file, and
/// for one that cannot be used a member that says so and stays offline.
pub fn members_of(found: &[Found]) -> Vec<Member> {
    found.iter().map(member_of).collect()
}

fn member_of(found: &Found) -> Member {
    let id = AccountId(found.id.clone());
    match &found.config {
        Ok(config) => Member::new(
            id,
            config.name.clone(),
            config.address.clone(),
            Box::new(RealBackend::start(config.clone())),
        ),
        Err(reason) => Member::failed(id, found.name.clone(), found.address.clone(), reason.clone()),
    }
}

#[cfg(test)]
include!("members.tests.rs");
