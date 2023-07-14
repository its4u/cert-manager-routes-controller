use crate::types::ContextData;
use crate::ISSUER_ANNOTATION_KEY;

use crate::crd::route::Route;
use crate::tools::{get_secret_tls_data, resource_to_string};
use kube::{
    api::{Patch, PatchParams},
    Api, ResourceExt,
};
use serde_json;
use std::fmt;

const TERMINATION: &'static str = "edge";
const REDIRECT_POLICY: &'static str = "Redirect";
pub const TLS_CRT: &'static str = "tls.crt";
pub const TLS_KEY: &'static str = "tls.key";
const CA_CRT: &'static str = "ca.crt";

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", resource_to_string(&self.name_any(), &self.namespace().unwrap()))
    }
}

pub fn is_valid_route(route: &Route) -> bool {
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
    let data = get_secret_tls_data(&cert_name, &ctx).await?;
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
