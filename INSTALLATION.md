# Installation and Configuration
## Using `docker compose` (Recommended method)
There is a sample `compose.yaml` file in this repository. It contains
everything needed for a basic install. You can use it as a base, modifying
it as needed. Run it with
```
docker compose up -d
```
If you're using a custom location for the `db_url`, make sure to make that file
before running the docker image, as otherwise a directory will be created in its
place, resulting in possibly unwanted behavior.

## Building and running with docker
### `docker run` method
0. (Only if you really want to) Build the image for the default `x86_64-unknown-linux-musl` target:
```
docker build . -t chhoto-url
```
For building on `arm64` or `arm/v7`, use the following:
```
docker build . -t chhoto-url --build-arg target=<desired-target>
```
Make sure that the desired target is a `musl` one, since the docker image is built from `scratch`.
For cross-compilation, take a look at the `Makefile`. It builds and pushes for `linux/amd64`, `linux/aarch64`
and `linux/arm/v7` architectures. For any other architectures, open a discussion, and I'll try to help you out.
1. Run the image
```
docker run -p 4567:4567
    -e password="password"
    -d chhoto-url:latest
```
1.a Make the database file available to host (optional)
```
touch ./urls.sqlite
docker run -p 4567:4567 \
    -e password="password" \
    -v ./urls.sqlite:/urls.sqlite \
    -e db_url=/urls.sqlite \
    -d chhoto-url:latest
```
1.b Further, set the URL of your website (optional)
```
touch ./urls.sqlite
docker run -p 4567:4567 \
    -e password="password" \
    -v ./urls.sqlite:/urls.sqlite \
    -e db_url=/urls.sqlite \
    -e site_url="https://www.example.com" \
    -d chhoto-url:latest
```
1.c Further, set an API key to activate JSON result mode (optional)

```
docker run -p 4567:4567 \
    -e password="password" \
    -e api_key="SECURE_API_KEY" \
    -v ./urls.sqlite:/urls.sqlite \
    -e db_url=/urls.sqlite \
    -e site_url="https://www.example.com" \
    -d chhoto-url:latest
```

You can set the redirect method to Permanent 308 (default) or Temporary 307 by setting
the `redirect_method` variable to `TEMPORARY` or `PERMANENT` (it's matched exactly). By
default, the auto-generated links are adjective-name pairs. You can use UIDs by setting
the `slug_style` variable to `UID`. You can also set the length of those slug by setting
the `slug_length` variable. It defaults to 8, and a minimum of 4 is supported.

You can provide hashed password and API key for extra security. Note that it will add some latency
to some of your requests and use more resources in general. The only supported algorithm for now is Argon2.
Recommended command for hashing:
```bash
echo -n <password> | argon2 <salt> -id -t 3 -m 16 -l 32 -e
```
You may also use online tools for this step.

To enable public mode, set `public_mode` to `Enable`. With this, anyone will be able to add 
links. Listing existing links or deleting links will need admin access using the password. If
`public_mode` is enabled, and `public_mode_expiry_delay` is set to a positive value, submitted links
will expire in that given time. The user can still choose a shorter expiry delay.
To completely disable the frontend, set `disable_frontend` to `True`.

By default, the server sends no Cache-Control headers. You can set custom `cache_control_header` 
to send your desired headers. It must be a comma separated list of valid 
[RFC 7234 §5.2](https://datatracker.ietf.org/doc/html/rfc7234#section-5.2) headers. For example,
you can set it to `no-cache, private` to disable caching. It might help during testing if
served through a proxy.

## Deploying in your Kubernetes cluster with Helm
The helm values are very sparse to keep it simple. If you need more values to be variable, feel free to adjust.

The PVC allocates 100Mi and the PV is using a host path volume.

The helm chart assumes you have [cert manager](https://github.com/jetstack/cert-manager) deployed to have TLS certificates managed easily in your cluster. Feel free to remove the issuer and adjust the ingress if you're on AWS with EKS for example. To install cert-manager, I recommend using the ["kubectl apply" way](https://cert-manager.io/docs/installation/kubectl/) to install cert-manager.

To get started, `cp helm-chart/values.yaml helm-chart/my-values.yaml` and adjust `password`, `fqdn` and `letsencryptmail` in your new `my-values.yaml`, then just run

``` bash
cd helm-chart
helm upgrade --install chhoto-url . -n chhoto-url --create-namespace -f my-values.yaml
```

## Browser extension
There's an (unofficial) browser extension maintained by @SolninjaA for shortening URLs easily using Chhoto URL. 
[You can take a look at it here.](https://github.com/SolninjaA/Chhoto-URL-Extension)

## OpenBSD package
There's an (unofficial) FreeBSD package maintained by @jcpsantiago for installing Chhoto URL.
[You can take a look at it here.](https://tangled.sh/@jcpsantiago.xyz/freebsd-ports/tree/main/www/chhoto-url)
Feel free to discuss any issues or suggestions in [#56](https://github.com/SinTan1729/chhoto-url/discussions/56).

