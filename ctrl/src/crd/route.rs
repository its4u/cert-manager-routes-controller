// WARNING: generated by kopium - manual changes will be overwritten
// kopium command: curl -sSL https://raw.githubusercontent.com/openshift/api/master/route/v1/route.crd.yaml | kopium -f - > src/route.rs
// kopium version: 0.15.0

use kube::CustomResource;
use serde::{Serialize, Deserialize};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(group = "route.openshift.io", version = "v1", kind = "Route", plural = "routes")]
#[kube(namespaced)]
#[kube(status = "RouteStatus")]
#[kube(schema = "disabled")]
pub struct RouteSpec {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "alternateBackends")]
    pub alternate_backends: Option<Vec<RouteAlternateBackends>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<RoutePort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<RouteTls>,
    pub to: RouteTo,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "wildcardPolicy")]
    pub wildcard_policy: Option<RouteWildcardPolicy>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteAlternateBackends {
    pub kind: RouteAlternateBackendsKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteAlternateBackendsKind {
    Service,
    #[serde(rename = "")]
    KopiumEmpty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutePort {
    #[serde(rename = "targetPort")]
    pub target_port: IntOrString,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteTls {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "caCertificate")]
    pub ca_certificate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "destinationCACertificate")]
    pub destination_ca_certificate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "insecureEdgeTerminationPolicy")]
    pub insecure_edge_termination_policy: Option<RouteTlsInsecureEdgeTerminationPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub termination: RouteTlsTermination,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteTlsInsecureEdgeTerminationPolicy {
    Allow,
    None,
    Redirect,
    #[serde(rename = "")]
    KopiumEmpty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteTlsTermination {
    #[serde(rename = "edge")]
    Edge,
    #[serde(rename = "reencrypt")]
    Reencrypt,
    #[serde(rename = "passthrough")]
    Passthrough,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteTo {
    pub kind: RouteToKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteToKind {
    Service,
    #[serde(rename = "")]
    KopiumEmpty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteWildcardPolicy {
    None,
    Subdomain,
    #[serde(rename = "")]
    KopiumEmpty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteStatus {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingress: Option<Vec<RouteStatusIngress>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteStatusIngress {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<RouteStatusIngressConditions>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "routerCanonicalHostname")]
    pub router_canonical_hostname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "routerName")]
    pub router_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "wildcardPolicy")]
    pub wildcard_policy: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteStatusIngressConditions {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastTransitionTime")]
    pub last_transition_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub status: String,
    #[serde(rename = "type")]
    pub r#type: String,
}
