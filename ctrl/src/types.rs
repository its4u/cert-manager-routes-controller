use kube::{Client, runtime::events::Recorder};

#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct ContextData {
    pub client: Client,
    pub cert_manager_namespace: String,
    pub recorder: Recorder,
}

impl ContextData {
    pub fn new(client: Client, cert_manager_namespace: String, recorder: Recorder) -> Self {
        Self {
            client,
            cert_manager_namespace,
            recorder,
        }
    }
}
