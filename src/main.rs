pub mod certificate;
pub mod crd;
pub mod route;
pub mod tools;
pub mod types;

use certificate::{annotate_cert, certificate_exists, create_certificate, is_cert_annotated};
use crd::{certificate::Certificate, route::Route};
use futures::StreamExt;
use kube::{
    api::ListParams,
    runtime::controller::{Action, Controller},
    runtime::reflector::ObjectRef,
    Api, Client, ResourceExt,
};
use route::{
    add_finalizer, is_tls_up_to_date, is_valid_route, populate_route_tls, remove_finalizer,
};
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
pub const FINALIZER: &'static str = "kubernetes";

/// The main function initializes the controller and runs it in a multi-threaded context.
///
/// The controller watches for [`Route`] and matching [`Certificate`] events.
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
        |obj| match obj.annotations().get(CERT_ANNOTATION_KEY) {
            Some(annotation) => annotation
                .split(",")
                .map(|s| {
                    if let Some((namespace, name)) = s.split_once("/") {
                        ObjectRef::new(name).within(namespace)
                    } else {
                        ObjectRef::new("")
                    }
                })
                .collect::<Vec<_>>(),
            None => vec![],
        },
    )
    .run(reconcile, error_policy, context)
    .for_each(|_| futures::future::ready(()))
    .await;
    Ok(())
}

/// The reconcile function is called for each [`Route`] event and related [`Certificate`] events by the main controller.
///
/// If the [`Route`] is being finalized or doesn't have the [`ISSUER_ANNOTATION_KEY`] annotation,
/// the route will be removed from the [`Certificate`] annotation if it exists.
///
/// Else, it checks if the [`Route`] is valid,
/// if a [`Certificate`] exists for the [`Route`]'s hostname,
/// if the [`Certificate`] is annotated with the [`Route`]'s name and namespace
/// and if the [`Route`]'s TLS is up to date.
///
/// This function is idempotent.
async fn reconcile(route: Arc<Route>, ctx: Arc<ContextData>) -> Result<Action, Error> {
    let mut remove_annotation: bool = false;

    if route.metadata.deletion_timestamp.is_some() && route.metadata.finalizers.as_ref().is_some() {
        remove_annotation = true;

        match remove_finalizer(&route, &ctx).await {
            Ok(_) => println!("Removed finalizer from Route `{}`", &route),
            Err(e) => {
                eprintln!("Error removing finalizer from Route `{}`: {}", &route, e);
                return Ok(Action::requeue(Duration::from_secs(
                    REQUEUE_ERROR_DURATION_SLOW,
                )));
            }
        }
    }

    if (remove_annotation || route.annotations().get(ISSUER_ANNOTATION_KEY).is_none())
        && route.spec.host.as_ref().is_some()
    {
        let cert_name = format_cert_name(&route.spec.host.as_ref().unwrap());
        if certificate_exists(&cert_name, &ctx).await
            && is_cert_annotated(&cert_name, &route, &ctx)
                .await
                .unwrap_or(true)
        {
            match annotate_cert(&cert_name, &route, &ctx, false).await {
                Ok(certificate) => println!(
                    "Removed  Route `{}` from Certificate `{}` annotation",
                    &route, &certificate
                ),
                Err(e) => {
                    eprintln!(
                        "Error removing Route `{}` from Certificate `{}/{}` annotation: {}",
                        &ctx.cert_manager_namespace, &cert_name, &route, e
                    );
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            }
        }
    } else if is_valid_route(&route) {
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
                        "Error creating Certificate `{}/{}` requested by Route `{}`: {}",
                        &ctx.cert_manager_namespace, &cert_name, &route, e
                    );
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            }
        }

        match is_tls_up_to_date(&route, &cert_name, &ctx).await {
            Ok(false) | Err(_) => match populate_route_tls(&route, &cert_name, &ctx).await {
                Ok(_) => println!("Populated TLS for Route `{}`", &route),
                Err(e) => {
                    eprintln!("Error populating TLS for Route `{}`: {}", &route, e);
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            },
            _ => {}
        }

        if !route.finalizers().contains(&FINALIZER.to_string()) {
            match add_finalizer(&route, &ctx).await {
                Ok(_) => println!("Added finalizer to Route `{}`", &route),
                Err(e) => {
                    eprintln!("Error adding finalizer to Route `{}`: {}", &route, e);
                    return Ok(Action::requeue(Duration::from_secs(
                        REQUEUE_ERROR_DURATION_SLOW,
                    )));
                }
            }
        }
    }

    // Ensure that each managed certificate is correclty annotated
    for route in Api::<Route>::all(ctx.client.clone())
        .list(&ListParams::default())
        .await
        .unwrap()
    {
        if is_valid_route(&route) {
            let cert_name = format_cert_name(&route.spec.host.as_ref().unwrap());
            match is_cert_annotated(&cert_name, &route, &ctx).await {
                Ok(false) | Err(_) => match annotate_cert(&cert_name, &route, &ctx, true).await {
                    Ok(certificate) => println!(
                        "Annotated Certificate `{}` for Route `{}`",
                        &certificate, &route
                    ),
                    Err(e) => {
                        eprintln!(
                            "Error annotating Certificate `{}/{}` requested by Route `{}`: {}",
                            &ctx.cert_manager_namespace, &cert_name, &route, e
                        );
                        return Ok(Action::requeue(Duration::from_secs(
                            REQUEUE_ERROR_DURATION_SLOW,
                        )));
                    }
                },
                _ => {}
            }
        }
    }

    Ok(Action::requeue(Duration::from_secs(
        REQUEUE_DEFAULT_INTERVAL,
    )))
}

/// The error policy function is called by the controller when an unexpected error occurs during the reconcile function.
fn error_policy(route: Arc<Route>, err: &Error, _ctx: Arc<ContextData>) -> Action {
    eprintln!("Error reconciling Route `{}`: {}", &route, err);
    Action::requeue(Duration::from_secs(REQUEUE_ERROR_DURATION_FAST))
}
