use kube::{Api, api::{ObjectMeta, PostParams}};

use crate::crd::certificate::{Certificate, CertificateSpec, CertificateIssuerRef, CertificatePrivateKey, CertificatePrivateKeyAlgorithm};
use crate::crd::route::Route;
use crate::types::{ContextData, Result};

const ISSUER_ANNOTATION: &'static str = "cert-manager.io/issuer";

async fn is_valid_route(route: &Route) -> bool {
    if route.spec.host == None
        || route.metadata.annotations == None 
        || route.metadata.annotations.as_ref().unwrap().get(ISSUER_ANNOTATION) == None {
        false
    } else {
        true
    }
}

async fn create_certificate (route: &Route, ctx: &ContextData) -> Result<(), kube::Error> {
    let annotations = route.metadata.annotations.as_ref().unwrap();
    let hostname = route.spec.host.as_ref().unwrap();
    let cert_name = format!("{}-cert", &hostname);
    let cert_api: Api<Certificate> = Api::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace);
    let cert = Certificate {
        status: None,
        metadata: ObjectMeta {
            name: Some(cert_name.clone()),
            namespace: Some(ctx.cert_manager_namespace.clone()),
            ..Default::default()
        },
        spec: CertificateSpec {
            secret_name: format!("{}-tls", &hostname),
            dns_names: Some(vec![hostname.clone()]),
            issuer_ref: CertificateIssuerRef {
                name: annotations.get(ISSUER_ANNOTATION).unwrap().to_owned(),
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
    cert_api.create(&PostParams::default(), &cert).await?;
    println!("Created Certificate `{}` in namespace {}", &cert_name, &ctx.cert_manager_namespace);
    Ok(())
}
