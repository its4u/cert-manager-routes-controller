use kube::Client;

#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct ContextData {
    pub client: Client,
    pub cert_manager_namespace: String,
    pub cert_annotation_key: String,
}

impl ContextData {
    pub fn new(client: Client, cert_manager_namespace: String, cert_annotation_key: String) -> Self {
        Self {
            client,
            cert_manager_namespace,
            cert_annotation_key
        }
    }
}
