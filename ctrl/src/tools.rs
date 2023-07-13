use crate::crd::{certificate::Certificate, route::Route};
use crate::route::{TLS_CRT, TLS_KEY};
use crate::types::ContextData;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use kube::{Api, ResourceExt};
use std::collections::BTreeMap;
use std::collections::HashSet;

pub fn route_to_string(route: &Route) -> String {
    format!("{}:{}", route.namespace().unwrap(), route.name_any())
}

pub fn format_cert_annotation(cert_annotation: Option<&String>, route: &Route) -> String {
    match cert_annotation {
        Some(cert_annotation) => {
            let mut annotations: HashSet<String> = HashSet::new();
            cert_annotation.split(",").for_each(|annotation| {
                annotations.insert(annotation.to_owned());
            });
            annotations.insert(route_to_string(&route));
            annotations.into_iter().collect::<Vec<String>>().join(",")
        }
        None => route_to_string(&route),
    }
}

pub fn format_cert_name(hostname: &str) -> String {
    format!("{}-cert", hostname)
}

pub fn format_secret_name(hostname: &str) -> String {
    format!("{}-tls", hostname)
}

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
