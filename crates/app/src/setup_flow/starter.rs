//! Starting the backend that tries an account. The real one is the engine's;
//! the tests of the flow use a stand-in that never touches a network.

use engine::{AccountConfig, RealBackend};
use model::MailBackend;

/// Starts the backend an account is tried on. The same backend is kept and
/// joins the running accounts once the server has let it in.
pub trait BackendStarter {
    fn start(&self, config: AccountConfig) -> Box<dyn MailBackend>;
}

/// The engine's real backend.
pub struct EngineBackends;

impl BackendStarter for EngineBackends {
    fn start(&self, config: AccountConfig) -> Box<dyn MailBackend> {
        Box::new(RealBackend::start(config))
    }
}
