# Installation and Configuration

## Using `docker compose` (Recommended method)

There is a sample `compose.yaml` file in this repository. It contains
everything needed for a basic install. The OCI image itself is built with
a GitHub action (starting from version 6.2.6), and you can [check the workflow for yourself](./.github/workflows/docker-release.yml)
and confirm that it's indeed built from source and nothing silly is going on.

The container images come in two flavors. The default one is made from scratch, and is as light as possible.
The tags with `-alpine` suffix are built on top of alpine, so are a little bit larger. But they have
the basic UNIX tools for debugging, so might be worth using in case you want to play around with the image.
The `dev` tags are always built on top of alpine. All of these images are available both on the Docker Hub (recommended)
and GHCR, except the `dev` builds which are only available on GHCR. All of these images are available for `linux/amd64`,
`linux/arm64`, and `linux/arm/v7` architectures on Linux. These should also work just fine with `podman`, or any other
container engine supporting OCI images.

You can use the [provided compose file](./compose.yaml) as a base, modifying it as needed. Run it with

```
docker compose up -d
```

If you're using a custom location for the `db_url`, and using WAL mode, make sure to mount a whole
directory instead of a folder. If this is not done, there will be a low, but non-zero chance of data corruption.

It should be possible to run Chhoto URL with pretty much anything that supports OCI images e.g. `docker`, `podman quadlets`
(the repo contains a sample `chhoto-url.container` file for using with `quadlets`.) etc. Official
support is only provided for `docker` and `podman`, but it should be trivial to convert the `compose.yaml` file to other formats. If you need help,
feel free to open a discussion.

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
For cross-compilation, take a look at the `Makefile`. It has instructions to build the architectures
mentioned above., For any other architectures, open a discussion, and I'll try to help you out.

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
    -v ./data:/data \
    -e db_url=/data/urls.sqlite \
    -d chhoto-url:latest
```

_Note: All of this pretty much works exactly the same if you replace `docker` with `podman`. In fact,
that's what I use for testing. A sample file for podman quadlets is provided at
[`chhoto-url.container`](./chhoto-url.container)_

## Configuration options

All the configuration is done using environmental variables. Here's a link of all supported ones. Please take
a look at the ones marked with a `#` as those are important, especially [`use_wal_mode`](#use_wal_mode-).

### `db_url` \#

Location for the database file. Take a look at [`use_wal_mode`](#use_wal_mode-) before you change it. Defaults to
`urls.sqlite`. It is highly recommended that you mount a named volume or directory at a location like `/data` and
use something like `/data/urls.sqlite` as `db_url`.
(Of course, the actual names being used don't really matter.)

### `password` \#

Provide a secure password. If kept empty, anyone can access the website. Note that password is not encrypted in
transport, so it's recommended to use a reverse proxy like `caddy` or `nginx`.

### `api_key`

Provide a secure API key. It'll be checked at start for security. If the API key is considered weak, a strong API
key will be generated and printed in the logs, but the weak one will be used for the time being.

Example Linux command for generating a secure API key: `tr -dc A-Za-z0-9 </dev/urandom | head -c 128`.

If no API key is provided, the website will still work, but it'll be a significantly worse experience if you try
to use Chhoto URL from the CLI.

### `use_wal_mode` \#

If set to `True`, enables [`WAL` journal mode](https://sqlite.org/wal.html). Any other value is ignored.
It's highly recommended that you enable it, but make sure that you mount either a whole directory, or a named
volume, and have the database inside it. DO NOT mount a single file, as there will be a small chance of partial
data loss in that case.

If this is enabled, there'll be a significant boost in performance under high load, since write will no longer block reads.
Also, automated backups of the database will be enabled. Otherwise, `DELETE` journal mode is used by default, along with
[`EXTRA` synchronous](https://sqlite.org/pragma.html#pragma_synchronous) pragma. In `WAL` mode, `FULL` synchronous pragma is
used instead.

In both cases, we have full ACID compliance, but it does cost a bit of performance. If you expect to see high throughput (in the
order of hundreds of read/writes per second), take a look at the `ensure_acid` configuration option.

### `ensure_acid`

By default, the database is
[ACID (i.e. Atomic, Consistent, Isolated, and Durable)](https://www.slingacademy.com/article/acid-properties-in-sqlite-why-they-matter).
If you'd like to let go of durability for an increase in throughput, set this to `False`. Any other value will be ignored.

This is done by setting the [synchronous pragma](https://sqlite.org/pragma.html#pragma_synchronous) to `FULL` in `WAL`
[journal mode](https://sqlite.org/pragma.html#pragma_journal_mode), and to `EXTRA` in `DELETE` journal mode.

_Note: There might be partial data loss only in case of system failure or power loss. Durability is maintained across application
crashes. If you do have data loss, you should only lose the data stored after the last sync with the database file. So, under normal
loads, you shouldn't lose any data anyway. But this is a real thing that can technically happen._

### `redirect_method` \#

Sets which redirection is used when a shortlink is resolved.

Can be set to `TEMPORARY` or `PERMANENT`, which will enable Temporary 307 or Permanent 308 redirects. Any other value
will be ignored, and a default of `PERMANENT` will be used.

### `slug_style`

Sets the style of slug used when auto-generating shortlinks.

Can be set to either `Pair` or `UID`. Any other value will be ignored, and a default value of `Pair` will be used.
In pair mode, adjective-name pairs are used for auto-generated links e.g. `gifted-ramanujan`. In UID mode, a randomly
generated slug is used.

### `slug_length`

If UID slugs are enabled, the length of the slug can be set using this. A minimum of 4 is supported, and it defaults to 16.
If you intend to have more than a few thousand shortlinks, it's strongly recommended that you use the UID `slug_style` with
a `slug_length` of 16 or more.

### `try_longer_slug`

If you do choose to use a short UID despite anticipating collisions, it's recommended that you set this to `True`.
In the event of a collision, this variable will result in a single retry attempt using a UID four digits longer than
`slug_length`. It has no effect for adjective-name slugs.

_Note: If not set, one retry will be attempted, just like adjective-name slugs. But it would use the same slug length._

### `listen_address`

The address Chhoto URL will bind to. Defaults to `0.0.0.0`.

Take a look at [this page](https://docs.rs/actix-web/4.11.0/actix_web/struct.HttpServer.html#method.bind)
for supported values and potential consequences. Changing `listen_address` is not recommended if
using docker.

### `port`

The port Chhoto URL will listen to. Defaults to `4567`.

### `allow_capital_letters`

If you want to use capital letters in the shortlink, set the `allow_capital_letters` variable to `True`. Any other
value is ignored.

This will also allow capital letters in UID slugs, if those are enabled. It has no effect for adjective-name slugs.

### `hash_algorithm` \#

If you want to provided hashed password and API Key, name a supported algorithm here. For now, the supported
values are: `Argon2`. More algorithms may be added later. Unsupported values are ignored.

_Note: If using a compose file, make sure to escape $ by $$._

_Note: It will add some latency to some of your requests and use more resources in general._

Recommended command for hashing:

```bash
echo -n <password> | argon2 <salt> -id -t 3 -m 16 -l 32 -e
```

You may also use online tools for this step.

### `public_mode`

To enable public mode, set `public_mode` to `Enable`. With this, anyone will be able to add
links. Listing existing links or deleting links will need admin access using the password. Any other values are
ignored.

### `public_mode_expiry_delay`

If `public_mode` is enabled, and `public_mode_expiry_delay` is set to a positive value, submitted links
will expire in that given time (in seconds). The user can still choose a shorter expiry delay.

It will have no effect for a logged in user i.e. the admin.

### `disable_frontend`

Set this to `True` to completely disable the frontend.

### `custom_landing_directory`

If you want to serve a custom landing page, put all your site related files, along with a valid `index.html` file in a
directory, and set this to the path of the directory. If using docker, you need to first
mount the directory inside the container. The admin page will then be located at `/admin/manage`.

### `cache_control_header`

By default, the server sends no Cache-Control headers. You can set custom headers here
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

```bash
cd helm-chart
helm upgrade --install chhoto-url . -n chhoto-url --create-namespace -f my-values.yaml
```
