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

1. Make sure that the chart repository is installed and up-to-date
```
helm repo add its4u-cm https://its4u.github.io/cert-manager-routes-controller
helm repo update
```

2. Install `cert-manager-routes-controller`

- In the default `cert-manager` namespace:

```
helm upgrade --install cert-manager-routes-controller its4u-cm/cert-manager-routes-controller
```

- In a custom `<CUSTOM_NS_NAME>` namespace:

```
helm upgrade --install cert-manager-routes-controller its4u-cm/cert-manager-routes-controller \
    --set cert_manager_namespace=<CUSTOM_NS_NAME>
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

## Where are the `Certificate`s stored?

All of the `Certificate`s and their respective `Secret` are stored in the same `CERT_MANAGER_NAMESPACE`. This allows us to reuse a `Certificate` cluster-wide and avoid reordering a `Certificate` that already exists in the cluster. 

> For instance, we have a route `https://example.com/hello` in the `hello` NS and a route `https://example.com/world` in the `world` NS. Both of these routes use the same domain, hence only one certificate is required. Therefore, we won't be ordering two certificates. We'll merely use the same one for both routes even though they're in a different namespace.

---

## Take a peek at our Wiki for more information

[The cert-manager-routes-controller wiki](https://github.com/its4u/cert-manager-routes-controller/wiki)
