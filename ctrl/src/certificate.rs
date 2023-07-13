use crate::crd::{
    certificate::{
        Certificate, CertificateIssuerRef, CertificatePrivateKey, CertificatePrivateKeyAlgorithm,
        CertificateSpec,
    },
    route::Route,
};
use crate::tools::{format_cert_annotation, format_cert_name, format_secret_name, route_to_string};
use crate::types::{ContextData, Result};
use crate::{CERT_ANNOTATION_KEY, ISSUER_ANNOTATION_KEY};
use kube::{
    api::{ObjectMeta, Patch, PatchParams, PostParams},
    core::PartialObjectMetaExt,
    Api,
};
use std::collections::HashSet;

pub async fn annotate_cert(
    cert_name: &String,
    route: &Route,
    ctx: &ContextData,
) -> Result<(), kube::Error> {
    let mut annotations =
        Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
            .get(cert_name)
            .await?
            .metadata
            .annotations
            .unwrap_or_default();
    let annotation = format_cert_annotation(annotations.get(CERT_ANNOTATION_KEY), &route);
    let _ = annotations.insert(CERT_ANNOTATION_KEY.to_owned(), annotation);
    let _ = Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .patch_metadata(
            cert_name,
            &PatchParams::default(),
            &Patch::Merge(
                &ObjectMeta {
                    annotations: Some(annotations),
                    ..Default::default()
                }
                .into_request_partial::<Certificate>(),
            ),
        )
        .await?;
    Ok(())
}

pub async fn create_certificate(
    route: &Route,
    ctx: &ContextData,
) -> Result<Certificate, kube::Error> {
    let annotations = route.metadata.annotations.as_ref().unwrap();
    let hostname = route.spec.host.as_ref().unwrap();
    let cert_name = format_cert_name(&hostname);
    let cert_api: Api<Certificate> =
        Api::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace);
    let cert = Certificate {
        status: None,
        metadata: ObjectMeta {
            name: Some(cert_name.clone()),
            namespace: Some(ctx.cert_manager_namespace.clone()),
            ..Default::default()
        },
        spec: CertificateSpec {
            secret_name: format_secret_name(&hostname),
            dns_names: Some(vec![hostname.clone()]),
            issuer_ref: CertificateIssuerRef {
                name: annotations.get(ISSUER_ANNOTATION_KEY).unwrap().to_owned(),
                kind: Some("ClusterIssuer".to_owned()),
                group: Some("cert-manager.io".to_owned()),
            },
            is_ca: Some(false),
            private_key: Some(CertificatePrivateKey {
                algorithm: Some(CertificatePrivateKeyAlgorithm::Ecdsa),
                encoding: None,
                rotation_policy: None,
                size: Some(256),
            }),
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
    };
    let cert = cert_api.create(&PostParams::default(), &cert).await?;
    Ok(cert)
}

pub async fn certificate_exists(cert_name: &str, ctx: &ContextData) -> bool {
    match Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .get(cert_name)
        .await
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

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
                        .contains(&route_to_string(&route)),
                )
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}
