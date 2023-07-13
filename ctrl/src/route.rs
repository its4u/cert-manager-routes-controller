use crate::types::ContextData;
use crate::ISSUER_ANNOTATION_KEY;

use crate::crd::{certificate::Certificate, route::Route};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams},
    Api, ResourceExt,
};
use serde_json;

const TERMINATION: &'static str = "edge";
const REDIRECT_POLICY: &'static str = "Redirect";
const TLS_CRT: &'static str = "tls.crt";
const TLS_KEY: &'static str = "tls.key";
const CA_CRT: &'static str = "ca.crt";

pub async fn is_valid_route(route: &Route) -> bool {
    if route.spec.host == None
        || route.metadata.annotations == None
        || route
            .metadata
            .annotations
            .as_ref()
            .unwrap()
            .get(ISSUER_ANNOTATION_KEY)
            == None
    {
        false
    } else {
        true
    }
}

pub async fn populate_route_tls(
    route: &Route,
    cert_name: &str,
    ctx: &ContextData,
) -> Result<(), kube::Error> {
    let certificate =
        Api::<Certificate>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
            .get(&cert_name)
            .await?;
    let secret = Api::<Secret>::namespaced(ctx.client.clone(), &ctx.cert_manager_namespace)
        .get(&certificate.spec.secret_name)
        .await?;
    let data = secret.data.as_ref().unwrap();
    if data.get(TLS_CRT) == None || data.get(TLS_KEY) == None {
        Err(kube::error::Error::Discovery(
            kube::error::DiscoveryError::MissingResource("tls".to_owned()),
        ))
    } else {
        let cert = std::str::from_utf8(&data.get(TLS_CRT).unwrap().0).unwrap();
        let key = std::str::from_utf8(&data.get(TLS_KEY).unwrap().0).unwrap();
        let ca = if data.get(CA_CRT) == None {
            None
        } else {
            Some(std::str::from_utf8(&data.get(TLS_CRT).unwrap().0).unwrap())
        };
        let routes = Api::<Route>::namespaced(ctx.client.clone(), &route.namespace().unwrap());
        let patch = serde_json::json!({
            "spec": {
                "tls": {
                    "termination": TERMINATION,
                    "insecureEdgeTerminationPolicy": REDIRECT_POLICY,
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
}
