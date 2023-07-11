#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct ContextData {
    pub client: kube::Client,
    pub cert_manager_namespace: String,
}
