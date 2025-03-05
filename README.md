<!-- SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

[![docker-pulls-badge](https://img.shields.io/docker/pulls/sintan1729/chhoto-url)](https://hub.docker.com/r/sintan1729/chhoto-url)
[![maintainer-badge](https://img.shields.io/badge/maintainer-SinTan1729-blue)](https://github.com/SinTan1729)
[![latest-release-badge](https://img.shields.io/github/v/release/SinTan1729/chhoto-url?label=latest%20release)](https://github.com/SinTan1729/chhoto-url/releases/latest)
![docker-image-size-badge](https://img.shields.io/docker/image-size/sintan1729/chhoto-url)
![commit-since-latest-release-badge](https://img.shields.io/github/commits-since/SinTan1729/chhoto-url/latest?sort=semver&label=commits%20since%20latest%20release)
[![license-badge](https://img.shields.io/github/license/SinTan1729/chhoto-url)](https://spdx.org/licenses/MIT.html)

# ![Logo](resources/assets/favicon-32.png) <span style="font-size:42px">Chhoto URL</span>

# What is it?
A simple selfhosted URL shortener with no unnecessary features. Simplicity
and speed are the main foci of this project. The docker image is ~6 MB (compressed),
and it uses <5 MB of RAM under regular use.

Don't worry if you see no activity for a long time. I consider this project
to be complete, not dead. I'm unlikely to add any new features, but I will try
and fix every bug you report. I will also try to keep it updated in terms of
security vulnerabilities.

If you feel like a feature is missing, please let me know by creating an issue
using the "feature request" template.

## But why another URL shortener?
Most URL shorteners are either bloated with unnecessary features, or are a pain to set up.
Even fewer are written with simplicity and lightness in mind. When I saw the `simply-shorten`
project (linked below), I really liked the idea but thought that it missed some features. Also,
I didn't like the fact that a simple app like this had a ~200 MB docker image (mostly due to the
included java runtime). So, I decided to rewrite it in Rust and add some features to it that I
thought were essential (e.g. hit counting).

## What does the name mean?
Chhoto (ছোট, [IPA](https://en.wikipedia.org/wiki/Help:IPA/Bengali): /tʃʰoʈo/) is the Bangla word
for small. URL means, well... URL. So the name simply means Small URL.

# Features
- Shortens URLs of any length to a randomly generated link.
- (Optional) Allows you to specify the shortened URL instead of the generated
  one. (It's surprisingly missing in a surprising number of alternatives.)
- Opening the shortened URL in your browser will instantly redirect you
  to the correct long URL. (So no stupid redirecting pages.)
- Super lightweight and snappy. (The docker image is only ~6MB and RAM uasge
  stays under 5MB under normal use.)
- Counts number of hits for each short link in a privacy respecting way
  i.e. only the hit is recorded, and nothing else.
- Has a mobile friendly UI, and automatic dark mode.
- Has a public mode, where anyone can add links without authentication. Deleting 
  or listing available links will need admin access using the password.
- Allows setting the URL of your website, in case you want to conveniently
  generate short links locally.
- Links are stored in an SQLite database.
- Available as a Docker container.
- Backend written in Rust using [Actix](https://actix.rs/), frontend
  written in plain HTML and vanilla JS, using [Pure CSS](https://purecss.io/)
  for styling.
- Uses very basic authentication using a provided password. It's not encrypted in transport.
  I  recommend using a reverse proxy such as [caddy](https://caddyserver.com/) to
  encrypt the connection by SSL.
  
# Bloat that will not be implemented
- Tracking or spying of any kind. The only logs that still exist are
 errors printed to stderr and the basic logging (only warnings) provided by the
 [`env_logger`](https://crates.io/crates/env_logger) crate.
- User management. If you need a shortener for your whole organization, either
 run separate containers for everyone or use something else.
- Cookies, newsletters, "we value your privacy" popups or any of the multiple
other ways modern web shows how anti-user it is. We all hate those, and they're
not needed here.
- Paywalls or messages begging for donations. If you want to support me (for
whatever reason), you can message me through GitHub issues.

# Screenshots 
#### Note: I'm using Dark Reader here to get the dark theme.
<p align="middle">
  <img src="screenshot-desktop.webp" height="250" alt="desktop screenshot" />
  <img src="screenshot-mobile.webp" height="250" alt="mobile screenshot" />
</p>

# Usage
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

To enable public mode, set `public_mode` to `Enable`. With this, anyone will be able to add 
links. Listing existing links or deleting links will need admin access using the password.

By default, the server sends no Cache-Control headers. You can set custom `cache_control_header` 
to send your desired headers. It must be a comma separated list of valid 
[RFC 7234 §5.2](https://datatracker.ietf.org/doc/html/rfc7234#section-5.2) headers. For example,
you can set it to `no-cache, private` to disable caching. It might help during testing if
served through a proxy.

## Instructions for CLI usage
The application can be used from the terminal using something like `curl`. In all the examples
below, replace `http://localhost:4567` with where your instance of `chhoto-url` is accessible.

You can get the version of `chhoto-url` the server is running using `curl http://localhost:4567/api/version` and
get the siteurl using `curl http://localhost:4567/api/siteurl`. These routes are accessible without any authentication.

### API key validation
**This is required for programs that rely on a JSON response from Chhoto URL**

In order to use API key validation, set the `api_key` environment variable. If this is not set, the API will default to cookie
validation (see section above). If the API key is insecure, a warning will be outputted along with a generated API key which may be used.

Example Linux command for generating a secure API key: `tr -dc A-Za-z0-9 </dev/urandom | head -c 128`

To add a link:
``` bash
curl -X POST -H "X-API-Key: <YOUR_API_KEY>" -d '{"shortlink":"<shortlink>", "longlink":"<longlink>"}' http://localhost:4567/api/new
```
Send an empty `<shortlink>` if you want it to be auto-generated. The server will reply with the generated shortlink.

To get information about a single shortlink:
``` bash
curl -H "X-API-Key: <YOUR_API_KEY>" -d '<shortlink>' http://localhost:4567/api/expand
```
(This route is not accessible using cookie validation.)

To get a list of all the currently available links:
``` bash
curl -H "X-API-Key: <YOUR_API_KEY>" http://localhost:4567/api/all
```

To delete a link:
``` bash
curl -X DELETE -H "X-API-Key: <YOUR_API_KEY>" http://localhost:4567/api/del/<shortlink>
```
Where `<shortlink>` is name of the shortened link you would like to delete. For example, if the shortened link is
`http://localhost:4567/example`, `<shortlink>` would be `example`.

The server will output when the instance is accessed over API, when an incorrect API key is received, etc.

### Cookie validation
If you have set up a password, first do the following to get an authentication cookie and store it in a file.
```bash
curl -X POST -d "<your-password>" -c cookie.txt http://localhost:4567/api/login
```
You should receive "Correct password!" if the provided password was correct. For any subsequent
request, please add `-b cookie.txt` to provide authentication.

To add a link, do
```bash
curl -X POST -d '{"shortlink":"<shortlink>", "longlink":"<longlink>"}' http://localhost:4567/api/new
```
Send an empty `<shortlink>` if you want it to be auto-generated. The server will reply with the generated shortlink.

To get a list of all the currently available links as `json`, do
```bash
curl http://localhost:4567/api/all
```

To delete a link, do
```bash
curl -X DELETE http://localhost:4567/api/del/<shortlink>
```
The server will send a confirmation.

## Disable authentication
If you do not define a password environment variable when starting the docker image, authentication
will be disabled.

This if not recommended in actual use however, as it will allow anyone to create new links and delete
old ones. This might not seem like a bad idea, until you have hundreds of links
pointing to illegal content. Since there are no logs, it's impossible to prove
that those links aren't created by you.

## Notes
- It started as a fork of [`simply-shorten`](https://gitlab.com/draganczukp/simply-shorten).
- There's an (unofficial) extension maintained by for shortening URLs easily using Chhoto URL.
  [You can take a look at it here.](https://github.com/SolninjaA/Chhoto-URL-Extension)
- The list of adjectives and names used for random short url generation is a modified
  version of [this list used by docker](https://github.com/moby/moby/blob/master/pkg/namesgenerator/names-generator.go).
