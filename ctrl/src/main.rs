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
use certificate::{annotate_cert, create_certificate, certificate_exists};
use types::*;
use tools::format_cert_name;

const REQUEUE_DEFAULT_INTERVAL: u64 = 3600;
const REQUEUE_ERROR_DURATION_SLOW: u64 = 120;
const REQUEUE_ERROR_DURATION_FAST: u64 = 5;
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

async fn reconcile(route: Arc<Route>, ctx: Arc<ContextData>) -> Result<Action> {
    if is_valid_route(&route).await {
        let route_name = route.name_any();
        let route_namespace = route.namespace().unwrap();
        let hostname = route.spec.host.as_ref().unwrap();
        let cert_name = format_cert_name(&hostname);

        println!("reconcile request from Route `{}:{}",  &route_namespace, &route_name);

        if !certificate_exists(&cert_name, &ctx).await {
            match create_certificate(&route, &ctx).await {
                Ok(_) => println!("Created Certificate `{}:{}` requested by Route `{}:{}`", &ctx.cert_manager_namespace, &cert_name, &route_namespace, &route_name),
                Err(e) => {
                    eprintln!("Error creating Certificate `{}:{}` requested by Route `{}:{}`: {}", &ctx.cert_manager_namespace, &cert_name, &route_namespace, &route_name, e);
                    return Ok(Action::requeue(Duration::from_secs(REQUEUE_ERROR_DURATION_SLOW)))
                }
            }
        }
        
        // populate route tls 

        match annotate_cert(&cert_name, &route, &ctx).await {
            Ok(_) => println!("Annotated Certificate `{}:{}` for Route `{}:{}`", &ctx.cert_manager_namespace, &cert_name, &route_namespace, &route_name),
            Err(e) => {
                eprintln!("Error annotating Certificate `{}:{}` requested by Route `{}:{}`: {}", &ctx.cert_manager_namespace, &cert_name, &route_namespace, &route_name, e);
                return Ok(Action::requeue(Duration::from_secs(REQUEUE_ERROR_DURATION_SLOW)));
            }
        }
    }
    Ok(Action::requeue(Duration::from_secs(REQUEUE_DEFAULT_INTERVAL)))
}

fn error_policy(_object: Arc<Route>, _err: &Error, _ctx: Arc<ContextData>) -> Action {
    Action::requeue(Duration::from_secs(REQUEUE_ERROR_DURATION_FAST))
}
