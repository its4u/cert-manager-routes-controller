# cert-manager OpenShift Routes controller

The power of `cert-manager` unleashed for OpenShift routes 🚀

## Requirements

An OpenShift Container Platform cluster with [`cert-manager`](https://cert-manager.io/) installed.

> We recommend that you use the [`cert-manager Operator for RedHat Openshift`](https://docs.openshift.com/container-platform/4.12/security/cert_manager_operator/index.html)

## Installation (Helm)

1. Add the chart repository

`helm repo add its4u-cm https://its4u.github.io/cert-manager-routes-controller`

2. Install the controller

- In the default `cert-manager` namespace:

`helm install cert-manager-routes-controller its4u-cm/cert-manager-routes-controller`

- In a custom `<CUSTOM_NS_NAME>` namespace:

`helm install cert-manager-routes-controller its4u-cm/cert-manager-routes-controller --set cert_manager_namespace=<CUSTOM_NS_NAME>`

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
    cert-manager.io/issuer: <CLUSTER_ISSUER_NAME>
```

3. Sit tight and watch your route's TLS being automatically populated!

> On the first certificate issuance, it might take a few minutes for the certificate to be ready. Hence, you might have to wait a little before you see your route being populated 😉

4. That's it! `cert-manager` will take care of the certificate renewal process. Our controller will ensure that your route's TLS is always populated with the correct up-to-date certificate.
