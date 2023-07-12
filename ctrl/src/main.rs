pub mod crd;
pub mod types;
pub mod tools;
pub mod route;
pub mod certificate;

use std::{sync::Arc, time::Duration};
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    runtime::controller::{Action, Controller},
    runtime::reflector::ObjectRef,
};
use crd::{route::Route, certificate::Certificate};
use route::is_valid_route;
use certificate::{annotate_cert, create_certificate};
use types::*;

pub const DEFAULT_CERT_MANAGER_NAMESPACE: &'static str = "cert-manager";
pub const CERT_MANAGER_NAMESPACE_ENV: &'static str = "CERT_MANAGER_NAMESPACE";
pub const CERT_ANNOTATION_KEY: &'static str = "cert-manager.io/routes";
pub const ISSUER_ANNOTATION_KEY: &'static str = "cert-manager.io/issuer";

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let cert_manager_namespace = std::env::var(CERT_MANAGER_NAMESPACE_ENV).unwrap_or(DEFAULT_CERT_MANAGER_NAMESPACE.to_owned());
    let context = Arc::new(
        ContextData::new(
            Client::try_default().await?, 
            cert_manager_namespace, 
        )
    );
    Controller::new(Api::<Route>::all(context.client.clone()), Default::default())
        .watches(
            Api::<Certificate>::all(context.client.clone()), 
            Default::default(),
            |obj| obj.annotations().get(CERT_ANNOTATION_KEY).unwrap_or(&String::from("")).split(",").map(|s| {
                let splitted = s.split_once(":");
                if splitted == None {
                    eprintln!("Invalid annotation value: {}", s);
                    ObjectRef::new("")
                } else {
                    let (namespace, name) = splitted.unwrap();
                    ObjectRef::new(name).within(namespace)
                }
            }).collect::<Vec<_>>()
        )
        .run(reconcile, error_policy, context)
        .for_each(|_| futures::future::ready(()))
        .await;
    Ok(())
}

async fn reconcile(obj: Arc<Route>, ctx: Arc<ContextData>) -> Result<Action> {
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Route>, _err: &Error, _ctx: Arc<ContextData>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
