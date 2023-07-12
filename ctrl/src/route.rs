use crate::crd::route::Route;
use crate::ISSUER_ANNOTATION_KEY;

pub async fn is_valid_route(route: &Route) -> bool {
    if route.spec.host == None
        || route.metadata.annotations == None
        || route.metadata.annotations.as_ref().unwrap().get(ISSUER_ANNOTATION_KEY) == None {
        false
    } else {
        true
    }
}
