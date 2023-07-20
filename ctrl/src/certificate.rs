use crate::crd::{
    certificate::{Certificate, CertificateIssuerRef, CertificateSpec},
    route::Route,
};
use crate::tools::{
    format_cert_annotation, format_cert_name, format_secret_name, resource_to_string,
};
use crate::types::ContextData;
use crate::{CERT_ANNOTATION_KEY, ISSUER_ANNOTATION_KEY};
use kube::{
    api::{ObjectMeta, Patch, PatchParams, PostParams},
    Api,
};
use std::collections::HashSet;
use std::fmt;

impl Certificate {
    /// Create a new [`Certificate`] with some default values.
    ///
    /// ### Arguments
    ///
    /// * `name` - The name of the [`Certificate`].
    /// * `hostname` - The dnsName of the [`Certificate`].
    /// * `issuer_name` - The `ClusterIssuer` to use for the [`Certificate`]
    /// * `ctx` - The [`ContextData`].
    ///
    /// ### Returns
    ///
    /// A new [`Certificate`] instance.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let cert = Certificate::new_default(&name, &hostname, &issuer_name, &ctx);
    /// println!("Created Certificate: {}", cert);
    /// ```
    fn new_default(
        name: &String,
        hostname: &String,
        issuer_name: &String,
        ctx: &ContextData,
    ) -> Self {
        Certificate {
            status: None,
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(ctx.cert_manager_namespace.clone()),
                ..Default::default()
            },
            spec: CertificateSpec {
                secret_name: format_secret_name(&hostname),
                dns_names: Some(vec![hostname.clone()]),
                issuer_ref: CertificateIssuerRef {
                    name: issuer_name.clone(),
                    kind: Some("ClusterIssuer".to_owned()),
                    group: Some("cert-manager.io".to_owned()),
                },
                is_ca: Some(false),
                private_key: None,
                additional_output_formats: None,
                common_name: None,
                duration: None,
                email_addresses: None,
                encode_usages_in_request: None,
                ip_addresses: None,
                keystores: None,
                literal_subject: None,
                renew_before: None,
                revision_history_limit: None,
                secret_template: None,
                subject: None,
                uris: None,
                usages: None,
            },
        }
    }
}

/// Implement the [`fmt::Display`] trait for a [`Certificate`].
/// It writes the data in [`resource_to_string()`] format.
impl fmt::Display for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            resource_to_string(
                self.metadata.name.as_ref().unwrap(),
                self.metadata.namespace.as_ref().unwrap()
            )
        )
    }
}

/// Annotate a [`Certificate`] with a [`Route`] that uses it.
///
/// The annotation key is the one contained in [`CERT_ANNOTATION_KEY`].
/// The annotation value is a comma-separated list of [`Route`]s each in the [`resource_to_string()`] format.
///
/// ### Arguments
///
/// * `cert_name` - The name of the [`Certificate`] to annotate.
/// * `route` - The [`Route`] that uses the [`Certificate`].
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing the annotated [`Certificate`] or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// let cert = annotate_cert(&cert_name, &route, &ctx).await?;
/// println!("Annotated Certificate: {}", cert);
/// ```
pub async fn annotate_cert(
    cert_name: &String,
    route: &Route,
    ctx: &ContextData,
    add: bool,
) -> Result<Certificate, kube::Error> {
    let mut annotations =
        Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
            .get(cert_name)
            .await?
            .metadata
            .annotations
            .unwrap_or_default();
    let annotation = format_cert_annotation(annotations.get(CERT_ANNOTATION_KEY), &route, add);
    let _ = annotations.insert(CERT_ANNOTATION_KEY.to_owned(), annotation);
    let cert = Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .patch(
            cert_name,
            &PatchParams::default(),
            &Patch::Merge(&serde_json::json!({
                "metadata": {
                    "annotations": annotations,
                }
            })),
        )
        .await?;
    Ok(cert)
}

/// Create a [`Certificate`] for a [`Route`]'s hostname.
///
/// ### Arguments
///
/// * `route` - The [`Route`] that will use the [`Certificate`].
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing the created [`Certificate`] or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// let cert = create_certificate(&route, &ctx).await?;
/// println!("Created Certificate: {}", cert);
/// ```
pub async fn create_certificate(
    route: &Route,
    ctx: &ContextData,
) -> Result<Certificate, kube::Error> {
    let annotations = route.metadata.annotations.as_ref().unwrap();
    let hostname = route.spec.host.as_ref().unwrap();
    let cert_name = format_cert_name(&hostname);
    let cert_api: Api<Certificate> =
        Api::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace);
    let cert = Certificate::new_default(
        &cert_name,
        &hostname,
        &annotations.get(ISSUER_ANNOTATION_KEY).unwrap().to_owned(),
        &ctx,
    );
    Ok(cert_api.create(&PostParams::default(), &cert).await?)
}

/// Check whether a [`Certificate`] exists.
///
/// ### Arguments
///
/// * `cert_name` - The name of the [`Certificate`] to check.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`bool`] indicating whether the [`Certificate`] exists.
///
/// ### Example
///
/// ```rust
/// let exists = certificate_exists(&cert_name, &ctx).await;
/// println!("Certificate `{}` exists: {}", &cert_name, exists);
/// ```
pub async fn certificate_exists(cert_name: &str, ctx: &ContextData) -> bool {
    match Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .get(cert_name)
        .await
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Check whether a [`Certificate`] is annotated with a [`Route`].
///
/// ### Arguments
///
/// * `cert_name` - The name of the [`Certificate`] to check.
/// * `route` - The [`Route`] to check in the annotation.
/// * `ctx` - The [`ContextData`].
///
/// ### Returns
///
/// A [`Result`] containing a [`bool`] indicating whether the [`Certificate`] is annotated with the [`Route`] or a [`kube::Error`].
///
/// ### Example
///
/// ```rust
/// let is_annotated = is_cert_annotated(&cert_name, &route, &ctx).await?;
/// println!("Certificate `{}` is annotated with Route `{}`: {}", &cert_name, &route, is_annotated);
/// ```
pub async fn is_cert_annotated(
    cert_name: &str,
    route: &Route,
    ctx: &ContextData,
) -> Result<bool, kube::Error> {
    let cert = Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .get(cert_name)
        .await?;
    match cert.metadata.annotations {
        Some(annotations) => {
            if let Some(annotation) = annotations.get(CERT_ANNOTATION_KEY) {
                Ok(
                    HashSet::<String>::from_iter(annotation.split(",").map(|s| s.to_owned()))
                        .contains(&route.to_string()),
                )
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}
