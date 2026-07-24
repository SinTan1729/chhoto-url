<!-- SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Installation and Configuration

## Using `docker compose` (Recommended method)

The repository includes a [sample `compose.yaml` file](../deploy/compose.yaml). It contains
everything needed for a basic deployment.
Starting with version 6.2.6, OCI images are built by a GitHub Actions workflow.
You can [check the workflow for yourself](../.github/workflows/docker-release.yml) to make sure that nothing silly is going on.

The container images come in two flavors. The default image is built from [`scratch`](https://hub.docker.com/_/scratch),
and is as light as possible. The tags with the `-alpine` suffix are built on top of [`alpine`](https://hub.docker.com/_/alpine).
These images are slightly larger, but include basic Unix tools that can be useful for debugging or for inspecting the container
interactively. The `dev` tags are always built on top of `alpine` and are intended for testing only; **do not use these for production
workloads.** The tags `latest` and `scratch` are synonymous, as are `latest-alpine` and `alpine`.

All of these images are available both on the Docker Hub (recommended) and GHCR, except for `dev` builds, which are published
only to GHCR. They are built for Linux on the `linux/amd64`, `linux/arm64`, `linux/arm/v7`, and `linux/riscv64` architectures.
The images should also work with Podman and other OCI-compatible container engines.

You can use the provided `compose` file as a base, modifying it as needed. Run it with

```
docker compose up -d
```

If you're using a custom location for the [`CHHOTO_DB_URL`](#chhoto_db_url), and using WAL mode, make sure to mount a whole
directory instead of a folder. If this is not done, there will be a low, but non-zero chance of data corruption.

It should be possible to run Chhoto URL with pretty much anything that supports OCI images e.g. `docker`, `podman quadlets`
(the repo contains a sample `chhoto-url.container` file for using with `quadlets`.) etc. Official
support is only provided for `docker` and `podman`, but it should be trivial to convert the `compose.yaml` file to other formats.
If you need help, feel free to open a discussion.

## Building and running with docker

### `docker run` method

0. (Only if you really want to) Build the image for the default `x86_64-unknown-linux-musl` target:

```
docker build -f build/Containerfile . -t chhoto-url
```

For building on `arm64`, `arm/v7`, or `riscv64`, use the following:

```
docker build -f build/Containerfile . -t chhoto-url --build-arg target=<desired-target>
```

Make sure that the desired target is a `musl` one, since the docker image is built from `scratch`.
For cross-compilation, take a look at the `Makefile`. It has instructions to build the architectures
mentioned above., For any other architectures, open a discussion, and I'll try to help you out.

1. Run the image

```
docker run -p 4567:4567 \
    -f build/Containerfile \
    -e CHHOTO_PASSWORD="password" \
    -d chhoto-url:latest
```

1.a Make the database file available to host (optional)

```
touch ./urls.sqlite
docker run -p 4567:4567 \
    -f build/Containerfile
    -e CHHOTO_PASSWORD="password" \
    -v ./data:/data \
    -e CHHOTO_DB_URL=/data/urls.sqlite \
    -d chhoto-url:latest
```

_Note: All of this pretty much works exactly the same if you replace `docker` with `podman`. In fact,
that's what I use for testing. A sample file for podman quadlets is provided at
[`chhoto-url.container`](../deploy/chhoto-url.container)_

## Configuration options

All the configuration is done using environmental variables. Here's a link of all supported ones. Please take
a look at the ones marked with a `#` as those are important, especially [`CHHOTO_SQLITE_USE_WAL_MODE`](#chhoto_sqlite_use_wal_mode).

<!-- prettier-ignore-start -->
<a id="chhoto_db_url"></a>
### `CHHOTO_DB_URL` \#
<!-- prettier-ignore-end -->

Location for the database file. Take a look at [`CHHOTO_SQLITE_USE_WAL_MODE`](#chhoto_sqlite_use_wal_mode) before you change it. Defaults to
`urls.sqlite`. It is highly recommended that you mount a named volume or directory at a location like `/data` and
use something like `/data/urls.sqlite` here.
(Of course, the actual names being used don't really matter.)

<!-- prettier-ignore-start -->
<a id="chhoto_password"></a>
### `CHHOTO_PASSWORD` \#
<!-- prettier-ignore-end -->

Provide a secure password. If kept empty, anyone can access the website. Note that password is not encrypted in
transport, so it's recommended to use a reverse proxy like `caddy` or `nginx`.

<!-- prettier-ignore-start -->
<a id="chhoto_site_url"></a>
### `CHHOTO_SITE_URL` \#
<!-- prettier-ignore-end -->

Change this to your public-facing URL. This is optional, as the link will work at any URL as long as Chhoto URL
is accessible there. This mostly enhances the frontend experience, as copying to clipboard, QR code generation will
use it if available.
Do not surround it using quotes. If you have any unicode characters, please use the punycode. It'll be automatically
converted to the correct unicode in the frontend.

### `CHHOTO_API_KEY`

Provide a secure API key. It'll be checked at start for security. If the API key is considered weak, a strong API
key will be generated and printed in the logs, but the weak one will be used for the time being.

Example Linux command for generating a secure API key: `tr -dc A-Za-z0-9 </dev/urandom | head -c 128`.

If no API key is provided, the website will still work, but it'll be a significantly worse experience if you try
to use Chhoto URL from the CLI.

<!-- prettier-ignore-start -->
<a id="chhoto_sqlite_use_wal_mode"></a>
### `CHHOTO_SQLITE_USE_WAL_MODE` \#
<!-- prettier-ignore-end -->

If set to `True`, enables [`WAL` journal mode](https://sqlite.org/wal.html). Any other value is ignored.
It's highly recommended that you enable it, but make sure that you mount either a whole directory, or a named
volume, and have the database inside it. DO NOT mount a single file, as there will be a small chance of partial
data loss in that case.

If this is enabled, there'll be a significant boost in performance under high load, since write will no longer block reads.
Otherwise, `DELETE` journal mode is used by default, along with [`EXTRA` synchronous](https://sqlite.org/pragma.html#pragma_synchronous)
pragma. In `WAL` mode, `FULL` synchronous pragma is used instead.

In both cases, we have full ACID compliance, but it does cost a bit of performance. If you expect to see high throughput (in the
order of hundreds of read/writes per second), take a look at the [`CHHOTO_SQLITE_ENSURE_ACID`](#chhoto_sqlite_ensure_acid) configuration option.

### `CHHOTO_SQLITE_ENSURE_ACID`

By default, the database is
[ACID (i.e. Atomic, Consistent, Isolated, and Durable)](https://www.slingacademy.com/article/acid-properties-in-sqlite-why-they-matter).
If you'd like to let go of durability for an increase in throughput, set this to `False`. Any other value will be ignored.

This is done by setting the [synchronous pragma](https://sqlite.org/pragma.html#pragma_synchronous) to `FULL` in `WAL`
[journal mode](https://sqlite.org/pragma.html#pragma_journal_mode), and to `EXTRA` in `DELETE` journal mode.

_Note: There might be partial data loss only in case of system failure or power loss. Durability is maintained across application
crashes. If you do have data loss, you should only lose the data stored after the last sync with the database file. So, under normal
loads, you shouldn't lose any data anyway. But this is a real thing that can technically happen._

<!-- prettier-ignore-start -->
<a id="chhoto_redirect_method"></a>
### `CHHOTO_REDIRECT_METHOD` \#
<!-- prettier-ignore-end -->

Sets which redirection is used when a shortlink is resolved.

Can be set to `TEMPORARY` or `PERMANENT`, which will enable Temporary 307 or Permanent 308 redirects. Any other value
will be ignored, and a default of `PERMANENT` will be used.

### `CHHOTO_SLUG_STYLE`

Sets the style of slug used when auto-generating shortlinks.

Can be set to either `Pair` or `UID`. Any other value will be ignored, and a default value of `Pair` will be used.
In pair mode, adjective-name pairs are used for auto-generated links e.g. `gifted-ramanujan`. In UID mode, a randomly
generated slug is used.

### `CHHOTO_SLUG_LENGTH`

If UID slugs are enabled, the length of the slug can be set using this. A minimum of 4 is supported, and it defaults to 16.
If you intend to have more than a few thousand shortlinks, it's strongly recommended that you use the UID
[`CHHOTO_SLUG_STYLE`](#chhoto_slug_style) with a [`CHHOTO_SLUG_LENGTH`](#chhoto_slug_length) of 16 or more.

### `CHHOTO_TRY_LONGER_SLUG`

If you do choose to use a short UID despite anticipating collisions, it's recommended that you set this to `True`.
In the event of a collision, this variable will result in a single retry attempt using a UID four digits longer than
[`CHHOTO_SLUG_LENGTH`](#chhoto_slug_length). For adjective-name slugs, a four digit suffix is added e.g.
`gifted-ramanujan-1729`.

_Note: If not set, one retry will be attempted, just like adjective-name slugs. But it would use the same slug length._

### `CHHOTO_LISTEN_ADDRESS`

The address Chhoto URL will bind to. Defaults to `0.0.0.0`.

Take a look at [this page](https://docs.rs/actix-web/4.11.0/actix_web/struct.HttpServer.html#method.bind)
for supported values and potential consequences. Changing [`CHHOTO_LISTEN_ADDRESS`](#chhoto_listen_address) is not
recommended if using docker.
If running locally, you may not be able to access Chhoto URL depending on your OS, browser configs etc. In that case,
try setting `::` as the listen address to also support IPV6.

### `CHHOTO_LISTEN_PORT`

The port Chhoto URL will listen to. Defaults to `4567`.

### `CHHOTO_ALLOW_CAPITAL_LETTERS`

If you want to use capital letters in the shortlink, set the [`CHHOTO_ALLOW_CAPITAL_LETTERS`](#chhoto_allow_capital_letters)
variable to `True`. Any other value is ignored.

This will also allow capital letters in UID slugs, if those are enabled. It has no effect for adjective-name slugs.

<!-- prettier-ignore-start -->
<a id="chhoto_hash_algorithm"></a>
### `CHHOTO_HASH_ALGORITHM` \#
<!-- prettier-ignore-end -->

If you want to provided hashed password and API Key, name a supported algorithm here. For now, the supported
values are: `Argon2`. More algorithms may be added later. Unsupported values are ignored.

_Note: If using a compose file, make sure to escape $ by $$._

_Warning: It will add some latency to some of your requests and use more resources in general._

Recommended command for hashing:

```bash
echo -n <password> | argon2 <salt> -id -t 3 -m 16 -l 32 -e
```

You may also use online tools for this step.

### `CHHOTO_PUBLIC_MODE`

To enable public mode, set [`CHHOTO_PUBLIC_MODE`](#chhoto_public_mode) to `Enable`. With this, anyone will be able to add
links. Listing existing links or deleting links will need admin access using the password. Any other values are
ignored.

### `CHHOTO_PUBLIC_MODE_EXPIRY_DELAY`

If [`CHHOTO_PUBLIC_MODE`](#chhoto_public_mode) is enabled, and [`CHHOTO_PUBLIC_MODE_EXPIRY_DELAY`](#chhoto_public_mode_expiry_delay)
is set to a positive value, submitted links will expire in that given time (in seconds). The user can still
choose a shorter expiry delay.

It will have no effect for a logged in user i.e. the admin.

### `CHHOTO_DISABLE_FRONTEND`

Set this to `True` to completely disable the frontend.

### `CHHOTO_CUSTOM_LANDING_DIRECTORY`

If you want to serve a custom landing page, put all your site related files, along with a valid `index.html` file in a
directory, and set this to the path of the directory. If using docker, you need to first
mount the directory inside the container. The admin page will then be located at `/admin/manage`.

_Warning: Put anything except the `index.html` file inside some directory, because naked filenames will be treated
as shortlinks._

### `CHHOTO_CACHE_CONTROL_HEADER`

By default, the server sends no Cache-Control headers. You can set custom headers here
to send your desired headers. It must be a comma separated list of valid
[RFC 7234 §5.2](https://datatracker.ietf.org/docs/html/rfc7234#section-5.2) headers. For example,
you can set it to `no-cache, private` to disable caching. It might help during testing if
served through a proxy.

### `CHHOTO_FRONTEND_PAGE_SIZE`

This can be used to set the number of items shown per page in the frontend. This does not have any effect on the backend code.
Defaults to 10.

### `CHHOTO_EXTRA_PROTOCOLS`

Use this to allow extra protocols for longlinks. By default, only `http`, `https`, `ftp`, and `magnet` links are allowed. It should be a comma
separated list e.g. `ftps,obsidian`. Malformed protocols will be skipped.

### `RUST_LOG`

It controls the level of logging.

Logging is done using the `env_logger` crate. No personal information of the end user is ever logged, nor will it ever supported. The
logging level can be controlled using the [`RUST_LOG`](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) variable. By default,
the value is set to the following.

```
warn,chhoto_url=info,actix_session::middleware=error
```

If you want a bit more logging, try this.

```
warn,chhoto_url=debug,actix_session::middleware=error
```

## Note of eventual deprecation of old config options

The config variables used to have different names up to commit 228eb7a, after which they were changed to adhere to norms for config variable
naming. The old names will keep working for now, but _it is highly recommended to migrate to the new variable names_ as support for these
will eventually be dropped in some future major release.

## Backups

Database backups are created during init, along with daily backups taken between 3am and 4am. The backup files are created in a directory
called `backups`, which is in the same directory as the database file. The backups are SQLite files, and are named using the original
database file name, and backup type. Daily backups have `.daily1`, `.daily2` etc. as the suffix, whereas the init backups have
`.init1`, `.init2` etc. as the suffix.

Backups are automatically purged, keeping up to 3 init backups, and 7 daily backups at any time. It's still recommended to keep your own
backups on top of these.

## Deploying in your Kubernetes cluster with Helm

The helm values are very sparse to keep it simple. If you need more values to be variable, feel free to adjust.

The PVC allocates 100Mi and the PV is using a host path volume.

The helm chart assumes you have [cert manager](https://github.com/jetstack/cert-manager) deployed to have TLS
certificates managed easily in your cluster. Feel free to remove the issuer and adjust the ingress if you're on
AWS with EKS for example. To install cert-manager, I recommend using the
["kubectl apply" way](https://cert-manager.io/docs/installation/kubectl/) to install cert-manager.

To get started, `cp deploy/helm-chart/values.yaml deploy/helm-chart/my-values.yaml` and adjust `password`, `fqdn`
and `letsencryptmail` in your new `my-values.yaml`, then just run.

[All the related files can be found here.](../deploy/helm-chart)

```bash
cd helm-chart
helm upgrade --install chhoto-url . -n chhoto-url --create-namespace -f my-values.yaml
```
