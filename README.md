# WIP: cert-manager OpenShift routes controller
The power of `cert-manger` unleashed for OpenShift routes

## Installation

If your `cert-manager` install is not located in the default `cert-manager` NS, you may specify a custom NS with the env variable `CERT_MANAGER_NAMESPACE`.

> WIP

## How to use

1. Create a `ClusterIssuer`
2. Annotate the `Route` that needs to be managed by `cert-manager` as follows:
```yaml
annotations:
    ...
    cert-manager.io/issuer: CLUSTER_ISSUER_NAME
```
3. Sit tight and watch your route's TLS being automatically populated
4. That's it! `cert-manager` will take care of the certificate renewal process. Our controller will ensure that your route's TLS is always populated with the correct up-to-date certificate.
