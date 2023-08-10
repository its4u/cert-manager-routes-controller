use crate::crd::route::{Route, RouteSpec, RouteTo, RouteToKind, RouteTlsTermination, RouteTlsInsecureEdgeTerminationPolicy};
use crate::tools::{format_route_update_annotation, get_secret_tls_data, resource_to_string};
use crate::types::ContextData;
use crate::{CLUSTER_ISSUER_ANNOTATION_KEY, FINALIZER};
use kube::api::ObjectMeta;
use kube::core::object::HasSpec;
use kube::{
    api::{Patch, PatchParams},
    Api, ResourceExt,
};
use serde_json;
use std::collections::BTreeMap;
use std::fmt;

const DEFAULT_TERMINATION: RouteTlsTermination = RouteTlsTermination::Edge;
const DEFAULT_INSECURE_EDGE_TERMINATION_POLICY: RouteTlsInsecureEdgeTerminationPolicy = RouteTlsInsecureEdgeTerminationPolicy::Redirect;
pub const TLS_CRT: &'static str = "tls.crt";
pub const TLS_KEY: &'static str = "tls.key";
const CA_CRT: &'static str = "ca.crt";
const ROUTE_UPDATE_ANNOTATION_KEY: &'static str = "cert-manager.io/updates";

impl Route {
    /// Create a new test [`Route`] with some default values.
    ///
    /// Implemented for testing purposes.
    ///
    /// ### Arguments
    ///
    /// * `name` - The name of the [`Route`].
    /// * `namespace` - The namespace of the [`Route`].
    /// * `hostname` - The host of the [`Route`].
    ///
    /// ### Returns
    ///
    /// A new [`Route`] instance.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let route = Route::new_default(&name, &namespace, &hostname);
    /// println!("Created Route: {}", route); // Created Route: namespace:name
    /// ```
    pub fn new_test_route(
        name: &String,
        namespace: &String,
        hostname: &String,
        cert_manager_issuer: Option<&String>,
        annotation_key: Option<&String>,
    ) -> Self {
        Route {
            status: None,
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace.clone()),
                annotations: match cert_manager_issuer {
                    Some(issuer) => {
                        let mut annotations = BTreeMap::new();
                        let _ = annotations.insert(annotation_key.unwrap().clone(), issuer.clone());
                        Some(annotations)
                    }
                    None => None,
                },
                ..Default::default()
            },
            spec: RouteSpec {
                host: Some(hostname.clone()),
                path: None,
                to: RouteTo {
                    kind: RouteToKind::Service,
                    name: "test".to_owned(),
                    weight: None,
                },
                port: None,
                tls: None,
                wildcard_policy: None,
                alternate_backends: None,
                subdomain: None,
            },
        }
    }
}

/// Implement the [`fmt::Display`] trait for a [`Route`].
/// It writes the data in [`resource_to_string()`] format.
impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            resource_to_string(&self.name_any(), &self.namespace().unwrap())
        )
    }
}

/// Check whether a [`Route`] is should be handled by the controller.
///
/// A [`Route`] is valid if it has a [`spec.host`] and an [`ISSUER_ANNOTATION_KEY`] annotation.
///
/// ### Arguments
///
/// * `route` - The [`Route`] to check.
///
/// ### Returns
///
/// A [`bool`] indicating whether the [`Route`] is valid.
///
/// ### Example
///
/// ```rust
/// let valid = is_valid_route(&route).await?;
/// println!("Valid Route: {}", valid);
/// ```
pub fn is_valid_route(route: &Route) -> bool {
    if route.spec.host == None
        || route.metadata.annotations == None
        || route
            .metadata
            .annotations
            .as_ref()
            .unwrap()
            .get(CLUSTER_ISSUER_ANNOTATION_KEY)
            == None
    {
        false
    } else {
        true
    }
}

#[test]
fn test_is_valid_route() {
    let route = Route::new_test_route(
        &"test_name".to_owned(),
        &"test_ns".to_owned(),
        &"test_host".to_owned(),
        None,
        None,
    );
    assert_eq!(is_valid_route(&route), false);
    let route = Route::new_test_route(
        &"test".to_owned(),
        &"test".to_owned(),
        &"test".to_owned(),
        Some(&"test".to_owned()),
        Some(&"foo".to_owned()),
    );
    assert_eq!(is_valid_route(&route), false);

    let route = Route::new_test_route(
        &"test".to_owned(),
        &"test".to_owned(),
        &"test".to_owned(),
        Some(&"test".to_owned()),
        Some(&CLUSTER_ISSUER_ANNOTATION_KEY.to_owned()),
    );
    assert_eq!(is_valid_route(&route), true);
}

/// Populate the TLS section of a [`Route`] with the data from a [`Certificate`].
///
/// ### Arguments
///
/// * `route` - The [`Route`] to populate.
/// * `cert_name` - The name of the [`Certificate`] to use.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing `()` or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// match populate_route_tls(&route, &cert_name, &ctx).await {
///     Ok(_) => println!("Route TLS populated"),
///     Err(e) => eprintln!("Error populating Route TLS: {}", e),
/// }
pub async fn populate_route_tls(
    route: &Route,
    cert_name: &str,
    ctx: &ContextData,
) -> Result<(), kube::Error> {
    let data = get_secret_tls_data(&cert_name, &ctx).await?;
    let cert = std::str::from_utf8(&data.get(TLS_CRT).unwrap().0).unwrap();
    let key = std::str::from_utf8(&data.get(TLS_KEY).unwrap().0).unwrap();
    let ca = if data.get(CA_CRT) == None {
        None
    } else {
        Some(std::str::from_utf8(&data.get(TLS_CRT).unwrap().0).unwrap())
    };
    let routes = Api::<Route>::namespaced(ctx.client.clone(), &route.namespace().unwrap());
    let (termination, insecure_edge_termination_policy) = match route.spec().tls.is_some() {
        true => (
            &route.spec().tls.as_ref().unwrap().termination,
            &route.spec().tls.as_ref().unwrap().insecure_edge_termination_policy
        ),
        false => (&DEFAULT_TERMINATION, &Some(DEFAULT_INSECURE_EDGE_TERMINATION_POLICY))
    };
    let patch = serde_json::json!({
        "metadata":{
            "annotations": {
                ROUTE_UPDATE_ANNOTATION_KEY: format_route_update_annotation(route.annotations().get(ROUTE_UPDATE_ANNOTATION_KEY))
            },
        },
        "spec": {
            "tls": {
                "termination": termination,
                "insecureEdgeTerminationPolicy": insecure_edge_termination_policy,
                "key": key,
                "certificate": cert,
                "caCertificate": ca.unwrap_or_default()
            }
        }
    });
    let _ = routes
        .patch(
            &route.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;
    Ok(())
}

/// Check whether a [`Route`]'s TLS is up to date the latest related [`Certificate`].
///
/// ### Arguments
///
/// * `route` - The [`Route`] to check.
/// * `cert_name` - The name of the [`Certificate`] to use.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing a [`bool`] indicating whether the [`Route`]'s TLS is up to date or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// let up_to_date = is_tls_up_to_date(&route, &cert_name, &ctx).await?;
/// println!("TLS up to date: {}", up_to_date);
/// ```
pub async fn is_tls_up_to_date(
    route: &Route,
    cert_name: &str,
    ctx: &ContextData,
) -> Result<bool, kube::Error> {
    let secret_data = get_secret_tls_data(cert_name, &ctx).await?;
    if let Some(tls) = route.clone().spec.tls {
        if tls.key == None || tls.certificate == None {
            return Ok(false);
        }
        if tls.certificate.unwrap()
            != std::str::from_utf8(&secret_data.get(TLS_CRT).unwrap().0).unwrap()
            || tls.key.unwrap()
                != std::str::from_utf8(&secret_data.get(TLS_KEY).unwrap().0).unwrap()
        {
            return Ok(false);
        }
        if secret_data.get(CA_CRT) != None {
            if tls.ca_certificate == None
                || tls.ca_certificate.unwrap()
                    != std::str::from_utf8(&secret_data.get(CA_CRT).unwrap().0).unwrap()
            {
                return Ok(false);
            }
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Add the [`FINALIZER`] to a [`Route`].
///
/// ### Arguments
///
/// * `route` - The [`Route`] to add the finalizer to.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing `()` or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// match add_finalizer(&route, &ctx).await {
///    Ok(_) => println!("Finalizer added to Route"),
///   Err(e) => eprintln!("Error adding finalizer to Route: {}", e),
/// }
/// ```
pub async fn add_finalizer(route: &Route, ctx: &ContextData) -> Result<(), kube::Error> {
    let routes = Api::<Route>::namespaced(ctx.client.clone(), &route.namespace().unwrap());
    let patch = serde_json::json!({
        "metadata":{
            "finalizers": [FINALIZER],
        }
    });
    let _ = routes
        .patch(
            &route.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;
    Ok(())
}

/// Remove the finalizers from a [`Route`].
///
/// ### Arguments
///
/// * `route` - The [`Route`] to remove the finalizers from.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing `()` or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// match remove_finalizer(&route, &ctx).await {
///    Ok(_) => println!("Finalizers removed from Route"),
///   Err(e) => eprintln!("Error removing finalizers from Route: {}", e),
/// }
/// ```
pub async fn remove_finalizer(route: &Route, ctx: &ContextData) -> Result<(), kube::Error> {
    let routes = Api::<Route>::namespaced(ctx.client.clone(), &route.namespace().unwrap());
    let patch = serde_json::json!({
        "metadata":{
            "finalizers": null,
        }
    });
    let _ = routes
        .patch(
            &route.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;
    Ok(())
}
