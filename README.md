<img src="https://raw.githubusercontent.com/its4u/cert-manager-routes-controller/main/img/openshift-cert-manager-logo.png" alt="cert-manager OpenShift controller" height=200 width=200 align="left" />

## An anti-anxiety pill against certificates renewal nightmares in OpenShift

No more sleep disorders... No more spending nights wondering whether a certificate has expired in your cluster...<br>
**The automation power of `cert-manager` is now unleashed for OpenShift routes** 🚀

----

## Requirements

An OpenShift Container Platform cluster with [`cert-manager`](https://cert-manager.io/) installed.

> We recommend that you use the [`cert-manager Operator for RedHat Openshift`](https://docs.openshift.com/container-platform/4.12/security/cert_manager_operator/index.html)

----

## Installation (Helm)

1. Add the chart repository

```
helm repo add its4u-cm https://its4u.github.io/cert-manager-routes-controller
```

2. Install the controller

- In the default `cert-manager` namespace:

```
helm install cert-manager-routes-controller its4u-cm/cert-manager-routes-controller
```

- In a custom `<CUSTOM_NS_NAME>` namespace:

```
helm install cert-manager-routes-controller its4u-cm/cert-manager-routes-controller --set cert_manager_namespace=<CUSTOM_NS_NAME>
```

----

## How to use

1. Create a `ClusterIssuer`

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: <CLUSTER_ISSUER_NAME>
spec:
    ...
```

2. Annotate the `Route` that needs to be managed by `cert-manager` as follows:

```yaml
annotations:
    cert-manager.io/cluster-issuer: <CLUSTER_ISSUER_NAME>
```

3. Sit tight and watch your route's TLS being automatically populated!

> On the first certificate issuance, it might take a few minutes for the certificate to be ready. Hence, you might have to wait a little before you see your route being populated 😉

4. That's it!<br>`cert-manager` will take care of the certificate renewal process.<br>Our controller will ensure that your route's TLS is always populated with the correct up-to-date certificate.

----

## How does it work?

> WIP: Add a graph

### Where are the `Certificate`s stored?

All of the `Certificate` and their `Secret`s are stored in the same `CERT_MANAGER_NAMESPACE`. This allows us to reuse a `Certificate` cluster-wide and avoid reordering a `Certificate` that already exists on the cluster. 

> For instance, we have a route `https://example.com/hello` in the `hello` NS and a route `https://example.com/world` in the `world` NS. Both of these routes use the same domain, hence only one certificate is required. Therefore, we won't be ordering two certificates. We'll merely use the same one for both routes even though they're in a different namespace.

### How does the controller handle a reconcile request?

The controller gets a reconcile request from a `Route` because it noticed a changed on a it or because its related `Certificate` was modified.

- If the route is being finalized:
  - The controller will update the `Route`'s related certificate
  - The controller will terminate the `Route` by removing its finalizer
- Else if the route is annotated with the `cert-manager.io/issuer` annotation
  - The controller will create a `Certificate` in the `CERT_MANAGER_NAMESPACE` if it doesn't exist yet
  - The controller will ensure that the `Route` is correctly populated with the latest up-to-date certificate
  - The controller will ensure that the `Route` has a finalizer to properly handle its deletion
- The controller will ensure that each `Certificate` is correctly annotated to point to each of the `Route`s that uses it
- Each request is requeued to an hour forward to ensure that no watch event is missed
