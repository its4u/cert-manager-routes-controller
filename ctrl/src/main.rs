pub mod crd;
pub mod manage_route;
pub mod types;

use std::{sync::Arc, time::Duration};
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    runtime::controller::{Action, Controller}
};
use k8s_openapi::api::core::v1::Secret;
use crd::route::Route;
use manage_route::*;
use types::*;

const DEFAULT_CERT_MANAGER_NAMESPACE: &'static str = "cert-manager";
const CERT_MANAGER_NAMESPACE_ENV: &'static str = "CERT_MANAGER_NAMESPACE";


#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let cert_manager_namespace = std::env::var(CERT_MANAGER_NAMESPACE_ENV).unwrap_or(DEFAULT_CERT_MANAGER_NAMESPACE.to_owned());
    let context = Arc::new(ContextData { 
        client: Client::try_default().await?,
        cert_manager_namespace,
    });

    Controller::new(Api::<Route>::all(context.client.clone()), Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

async fn reconcile(obj: Arc<Route>, ctx: Arc<ContextData>) -> Result<Action> {
    println!("reconcile request: {}\nAnnotations: {:?}",  obj.name_any(), obj.metadata.annotations);
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Route>, _err: &Error, _ctx: Arc<ContextData>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
