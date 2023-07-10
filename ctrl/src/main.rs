pub mod crd;

use std::{sync::Arc, time::Duration};
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    runtime::controller::{Action, Controller}
};
use k8s_openapi::api::core::v1::Secret;
use crd::certificate::Certificate;
use crd::route::Route;


#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let client = Client::try_default().await?;
    let routes = Api::<Route>::all(client);

    Controller::new(routes.clone(), Default::default())
        .run(reconcile, error_policy, Arc::new(()))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

async fn reconcile(obj: Arc<Route>, ctx: Arc<()>) -> Result<Action> {
    println!("reconcile request: {}", obj.name_any());
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Route>, _err: &Error, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
