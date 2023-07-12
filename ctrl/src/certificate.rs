use kube::{Api, api::{ObjectMeta, PostParams, PatchParams, Patch}, core::PartialObjectMetaExt, ResourceExt};
use crate::crd::{
    route::Route, 
    certificate::{Certificate, CertificateSpec, CertificateIssuerRef, CertificatePrivateKey, CertificatePrivateKeyAlgorithm}
};
use crate::types::{ContextData, Result};
use crate::tools::{format_cert_annotation, format_cert_name, format_secret_name};
use crate::{CERT_ANNOTATION_KEY, ISSUER_ANNOTATION_KEY};

pub async fn annotate_cert(cert_name: &String, route: &Route, ctx: &ContextData) -> Result<(), kube::Error> {
    let mut annotations = route.metadata.annotations.clone().unwrap_or_default();
    let route_name = route.name_any();
    let route_namespace = route.namespace().unwrap();
    let annotation = format_cert_annotation(annotations.get(CERT_ANNOTATION_KEY), &route_name, &route_namespace);
    let _ = annotations.insert(CERT_ANNOTATION_KEY.to_owned(), annotation);
    let _ = Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace).patch_metadata(
        cert_name,
        &PatchParams::default(), 
        &Patch::Merge(&ObjectMeta {
            annotations: Some(annotations),
            ..Default::default()
        }.into_request_partial::<Certificate>())).await?;
    println!("Annotated Certificate `{}` in NS `{}`", &cert_name, &ctx.cert_manager_namespace);
    Ok(())
}

pub async fn create_certificate(route: &Route, ctx: &ContextData) -> Result<Certificate, kube::Error> {
    let annotations = route.metadata.annotations.as_ref().unwrap();
    let hostname = route.spec.host.as_ref().unwrap();
    let cert_name = format_cert_name(&hostname);
    let cert_api: Api<Certificate> = Api::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace);
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
                size: Some(256)
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
    println!("Created Certificate `{}` in namespace {}", &cert_name, &ctx.cert_manager_namespace);
    Ok(cert)
}
