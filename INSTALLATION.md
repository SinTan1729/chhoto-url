# Installation and Configuration
## Using `docker compose` (Recommended method)
There is a sample `compose.yaml` file in this repository. It contains
everything needed for a basic install. The OCI image itself is built with
a GitHub action (starting from version 6.2.6), and you can [check the workflow for yourself](./.github/workflows/docker-release.yml)
and confirm that it's indeed built from source and nothing silly is going on.

The images come in two flavors. The default one is made from scratch, and is as light as possible.
The tags with `-alpine` suffix are built on top of alpine, so are a little bit larger. But they have
the basic UNIX tools for debugging, so might be worth using in case you want to play around with the image.
The `dev` tags are always built on top of alpine. All of these images are available both on the Docker Hub (recommended)
and GHCR, except the `dev` builds which are only available on GHCR.

You can use the compose file as a base, modifying it as needed. Run it with
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

Example Linux command for generating a secure API key: `tr -dc A-Za-z0-9 </dev/urandom | head -c 128`.

You can set the redirect method to Permanent 308 (default) or Temporary 307 by setting
the `redirect_method` variable to `TEMPORARY` or `PERMANENT` (it's matched exactly). By
default, the auto-generated links are adjective-name pairs. You can use UIDs by setting
the `slug_style` variable to `UID`. You can also set the length of those slug by setting
the `slug_length` variable. It defaults to 8, and a minimum of 4 is supported. If you 
intend to have more than a few thousand shortlinks, it's strongly recommended that you 
use the UID `slug_style` with a `slug_length` of 16 or more.

If you want to use capital letters in the shortlink, set the `allow_capital_letters` variable
to `True`. This will also allow capital letters in UID slugs, if it is enabled.

If you do choose to use a short UID despite anticipating collisions, set `try_longer_slug` to `True`. 
In the event of a collision, this variable will result in a single retry attempt using 
a UID four digits longer than `slug_length`. It has no effect for adjective-name slugs.

Although it's unlikely, it's possible that your database is mangled after some update. 
For mission critical use cases, it's recommended to keep regular versioned backups of 
the database, and sticking to a minor release tag e.g. 5.8. You can either bind mount a file
for the database as described in 1.a above, or take a backup of the docker volume.

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
To completely disable the frontend, set `disable_frontend` to `True`. If you want to serve a custom
landing page, put all your site related files, along with an `index.html` file in a directory, and
set `custom_landing_directory` to the path of the directory. If using docker, you need to first
mount the directory inside the container. The admin page will then be located at `/admin/manage`.

By default, the server sends no Cache-Control headers. You can set custom `cache_control_header` 
to send your desired headers. It must be a comma separated list of valid 
[RFC 7234 §5.2](https://datatracker.ietf.org/doc/html/rfc7234#section-5.2) headers. For example,
you can set it to `no-cache, private` to disable caching. It might help during testing if
served through a proxy.

## Deploying in your Kubernetes cluster with Helm
The helm values are very sparse to keep it simple. If you need more values to be variable, feel free to adjust.

The PVC allocates 100Mi and the PV is using a host path volume.

The helm chart assumes you have [cert manager](https://github.com/jetstack/cert-manager) deployed to have TLS 
certificates managed easily in your cluster. Feel free to remove the issuer and adjust the ingress if you're on 
AWS with EKS for example. To install cert-manager, I recommend using the
["kubectl apply" way](https://cert-manager.io/docs/installation/kubectl/) to install cert-manager.

To get started, `cp helm-chart/values.yaml helm-chart/my-values.yaml` and adjust `password`, `fqdn`
and `letsencryptmail` in your new `my-values.yaml`, then just run
``` bash
cd helm-chart
helm upgrade --install chhoto-url . -n chhoto-url --create-namespace -f my-values.yaml
```

