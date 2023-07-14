pub mod certificate;
pub mod crd;
pub mod route;
pub mod tools;
pub mod types;

use certificate::{annotate_cert, certificate_exists, create_certificate, is_cert_annotated};
use crd::{certificate::Certificate, route::Route};
use futures::StreamExt;
use kube::{
    runtime::controller::{Action, Controller},
    runtime::reflector::ObjectRef,
    Api, Client, ResourceExt,
};
use route::{is_tls_up_to_date, is_valid_route, populate_route_tls};
use std::{sync::Arc, time::Duration};
use tools::format_cert_name;
use types::*;

const REQUEUE_DEFAULT_INTERVAL: u64 = 3600;
const REQUEUE_ERROR_DURATION_SLOW: u64 = 120;
const REQUEUE_ERROR_DURATION_FAST: u64 = 5;
pub const DEFAULT_CERT_MANAGER_NAMESPACE: &'static str = "cert-manager";
pub const CERT_MANAGER_NAMESPACE_ENV: &'static str = "CERT_MANAGER_NAMESPACE";
pub const CERT_ANNOTATION_KEY: &'static str = "cert-manager.io/routes";
pub const ISSUER_ANNOTATION_KEY: &'static str = "cert-manager.io/issuer";

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let cert_manager_namespace = std::env::var(CERT_MANAGER_NAMESPACE_ENV)
        .unwrap_or(DEFAULT_CERT_MANAGER_NAMESPACE.to_owned());
    let context = Arc::new(ContextData::new(
        Client::try_default().await?,
        cert_manager_namespace,
    ));
    Controller::new(
        Api::<Route>::all(context.client.clone()),
        Default::default(),
    )
    .watches(
        Api::<Certificate>::all(context.client.clone()),
        Default::default(),
        |obj| {
            obj.annotations()
                .get(CERT_ANNOTATION_KEY)
                .unwrap_or(&String::from(""))
                .split(",")
                .map(|s| {
                    let splitted = s.split_once(":");
                    if splitted == None {
                        eprintln!("Invalid annotation value: {}", s);
                        ObjectRef::new("")
                    } else {
                        let (namespace, name) = splitted.unwrap();
                        ObjectRef::new(name).within(namespace)
                    }
                })
                .collect::<Vec<_>>()
        },
    )
    .run(reconcile, error_policy, context)
    .for_each(|_| futures::future::ready(()))
    .await;
    Ok(())
}

async fn reconcile(route: Arc<Route>, ctx: Arc<ContextData>) -> Result<Action, Error> {    
    if is_valid_route(&route) {
        let hostname = route.spec.host.as_ref().unwrap();
        let cert_name = format_cert_name(&hostname);

        if !certificate_exists(&cert_name, &ctx).await {
            match create_certificate(&route, &ctx).await {
                Ok(certificate) => println!(
                    "Created Certificate `{}` requested by Route `{}`",
                    &certificate, &route
                ),
                Err(e) => {
                    eprintln!(
                        "Error creating Certificate `{}:{}` requested by Route `{}`: {}",
                        &ctx.cert_manager_namespace, &cert_name, &route, e
                    );
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            }
        }

        match is_cert_annotated(&cert_name, &route, &ctx).await {
            Ok(false) | Err(_) => match annotate_cert(&cert_name, &route, &ctx).await {
                Ok(certificate) => println!(
                    "Annotated Certificate `{}` for Route `{}`",
                    &certificate, &route
                ),
                Err(e) => {
                    eprintln!(
                        "Error annotating Certificate `{}:{}` requested by Route `{}`: {}",
                        &ctx.cert_manager_namespace, &cert_name, &route, e
                    );
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            },
            _ => {}
        }

        match is_tls_up_to_date(&route, &cert_name, &ctx).await {
            Ok(false) | Err(_) => match populate_route_tls(&route, &cert_name, &ctx).await {
                Ok(_) => println!(
                    "Populated TLS for Route `{}`",
                    &route
                ),
                Err(e) => {
                    eprintln!(
                        "Error populating TLS for Route `{}`: {}",
                        &route, e
                    );
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            },
            _ => {}
        }
    }

    Ok(Action::requeue(Duration::from_secs(
        REQUEUE_DEFAULT_INTERVAL,
    )))
}

fn error_policy(route: Arc<Route>, err: &Error, _ctx: Arc<ContextData>) -> Action {
    eprintln!(
        "Error reconciling Route `{}`: {}",
        &route,
        err
    );
    Action::requeue(Duration::from_secs(REQUEUE_ERROR_DURATION_FAST))
}
