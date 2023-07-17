use crate::crd::{certificate::Certificate, route::Route};
use crate::route::{TLS_CRT, TLS_KEY};
use crate::types::ContextData;
use chrono::Utc;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use kube::Api;
use std::collections::BTreeMap;
use std::collections::HashSet;

/// Format a resource to a string in the format `namespace:name`.
///
/// ### Arguments
///
/// * `name` - The name of the resource.
/// * `namespace` - The namespace of the resource.
///
/// ### Returns
///
/// A [`String`] containing the formatted resource.
///
/// ### Example
///
/// ```rust
/// let resource = resource_to_string("world", "hello");
/// println!("{}", resource); // hello/world
/// ```
pub fn resource_to_string(name: &str, namespace: &str) -> String {
    format!("{}/{}", namespace, name)
}

#[test]
fn test_resource_to_string() {
    assert_eq!(resource_to_string("name", "namespace"), "namespace/name");
    assert_ne!(resource_to_string("name", "namespace"), "name/namespace");
}

/// Format a [`Certificate`] annotation value in the format `namespace:name(,namespace:name)*`.
///
/// If the annotation doesn't exists yet, the annotation value will contain a single route.
/// Else, the route will be appended to the annotation value.
///
/// ### Arguments
///
/// * `cert_annotation` - The current annotation value.
/// * `route` - The [`Route`] to append to the annotation value.
///
/// ### Returns
///
/// A [`String`] containing the formatted annotation value.
///
/// ### Example
///
/// ```rust
/// let annotation = format_cert_annotation(None, &route);
/// println!("{}", annotation);
/// ```
pub fn format_cert_annotation(cert_annotation: Option<&String>, route: &Route, add: bool) -> String {
    match cert_annotation {
        Some(cert_annotation) if cert_annotation.is_empty() && add => route.to_string(),
        None if add => route.to_string(),
        Some(cert_annotation) => {
            let mut annotations: HashSet<String> = HashSet::new();
            cert_annotation.split(",").for_each(|annotation| {
                annotations.insert(annotation.to_owned());
            });
            if add {
                annotations.insert(route.to_string());
            } else {
                annotations.remove(&route.to_string());
            }
            annotations.into_iter().collect::<Vec<String>>().join(",")
        }
        _ => String::new(),
    }
}

#[test]
fn test_format_cert_annotation() {
    let route = Route::new_test_route(
        &"world".to_owned(),
        &"hello".to_owned(),
        &"example.com".to_owned(),
        None,
        None,
    );
    assert_eq!(format_cert_annotation(None, &route, true), "hello/world");
    assert_eq!(
        format_cert_annotation(Some(&"".to_owned()), &route, true),
        "hello/world"
    );
    assert_eq!(
        format_cert_annotation(Some(&"foo/bar".to_owned()), &route, true),
        "foo/bar,hello/world"
    );
    assert_eq!(
        format_cert_annotation(Some(&"foo/bar,alice/bob".to_owned()), &route, true),
        "foo/bar,alice/bob,hello/world"
    );
    assert_eq!(format_cert_annotation(None, &route, false), "");
    assert_eq!(
        format_cert_annotation(Some(&"".to_owned()), &route, false),
        ""
    );
    assert_eq!(
        format_cert_annotation(Some(&"foo/bar".to_owned()), &route, false),
        "foo/bar"
    );
    assert_eq!(
        format_cert_annotation(Some(&"foo/bar,hello/world".to_owned()), &route, false),
        "foo/bar"
    );
}

/// Format a [`Certificate`] name in the format `hostname-cert`.
///
/// ### Arguments
///
/// * `hostname` - The hostname of the [`Certificate`].
///
/// ### Returns
///
/// A [`String`] containing the formatted [`Certificate`] name.
///
/// ### Example
///
/// ```rust
/// let cert_name = format_cert_name("example.com");
/// println!("{}", cert_name); // example.com-cert
/// ```
pub fn format_cert_name(hostname: &str) -> String {
    format!("{}-cert", hostname)
}

#[test]
fn test_format_cert_name() {
    assert_eq!(format_cert_name("example.com"), "example.com-cert");
    assert_ne!(format_cert_name("example.com"), "example.com");
}

/// Format a [`Secret`] name in the format `hostname-tls`.
///
/// ### Arguments
///
/// * `hostname` - The hostname of the [`Secret`].
///
/// ### Returns
///
/// A [`String`] containing the formatted [`Secret`] name.
///
/// ### Example
///
/// ```rust
/// let cert_name = format_secret_name("example.com");
/// println!("{}", cert_name); // example.com-tls
/// ```
pub fn format_secret_name(hostname: &str) -> String {
    format!("{}-tls", hostname)
}

#[test]
fn test_format_secret_name() {
    assert_eq!(format_secret_name("example.com"), "example.com-tls");
    assert_ne!(format_secret_name("example.com"), "example.com");
}

/// Get the TLS data from a [`Secret`]'s [`Certificate`].
///
/// ### Arguments
///
/// * `cert_name` - The name of the [`Certificate`] to extract the TLS data from.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing a [`BTreeMap`] of the TLS data or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// let tls_data = get_secret_tls_data(&cert_name, &ctx).await?;
/// println!("TLS data: {:?}", tls_data);
/// ```
pub async fn get_secret_tls_data(
    cert_name: &str,
    ctx: &ContextData,
) -> Result<BTreeMap<std::string::String, ByteString>, kube::Error> {
    let certificate =
        Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
            .get(&cert_name)
            .await?;
    let secret = Api::<Secret>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .get(&certificate.spec.secret_name)
        .await?;
    let data = secret.data.unwrap();
    if data.get(TLS_CRT) == None || data.get(TLS_KEY) == None {
        Err(kube::error::Error::Discovery(
            kube::error::DiscoveryError::MissingResource("tls".to_owned()),
        ))
    } else {
        Ok(data)
    }
}

/// Format a [`Route`] update annotation value in the format `timestamp(,timestamp)*`.
///
/// If the annotation doesn't exists yet, the annotation value will contain a single timestamp.
/// Else, the timestamp will be prepended to the annotation value.
///
/// ### Arguments
///
/// * `annotation` - The current annotation value.
///
/// ### Returns
///
/// A [`String`] containing the formatted annotation value.
///
/// ### Example
///
/// ```rust
/// let annotation = format_route_update_annotation(None);
/// println!("{}", annotation); // timestamp of the current time
/// ```
pub fn format_route_update_annotation(annotation: Option<&String>) -> String {
    match annotation {
        Some(annotation) => format!("{},{}", Utc::now(), annotation),
        None => format!("{}", Utc::now()),
    }
}
