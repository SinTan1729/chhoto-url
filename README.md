![Docker Pulls](https://img.shields.io/docker/pulls/draganczukp/simply-shorten?style=for-the-badge)
![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/draganczukp/simply-shorten?style=for-the-badge)
![GitHub issues](https://img.shields.io/github/issues/draganczukp/simply-shorten?style=for-the-badge)

# What is it?
A simple selfhosted URL shortener with no unnecessary features.

If you feel
like a feature is missing, please let me know by creating an issue using the
"feature request" template.


## But why another URL shortener?
I've looked at a couple popular URL shorteners, however they either have
unnecessary features, or they didn't have all the features I wanted.

# Features
- Shortens URLs of any length to a fixed length, randomly generated string
- (Optional) Allows you to specify the shortened URL instead of the generated
  one (Missing in a surprising number of alternatives)
- Opening the fixed length URL in your browser will instantly redirect you
  to the correct long URL (you'd think that's a standard feature, but
  apparently it's not)
- Provides a simple API for adding new short links
- Links are stored in an SQLite database
- Available as a Docker container
- Backend written in Java using [Spark Java](http://sparkjava.com/), frontend
  written in plain HTML and vanilla JS, using [Pure CSS](https://purecss.io/)
  for styling
  
# Bloat that will not be implemented
- Logging, tracking or spying of any kind. The only logs that still exist are
 errors printed to stderr and the default SLF4J warning.
- User management. If you need a shortener for your whole organisation, either
 run separate containers for everyone or use something else.
- Cookies, newsletters, "we value your privacy" popups or any of the multiple
other ways modern web shows how anti-user it is. We all hate those, and they're
not needed here.
- Paywalls or messages begging for donations. If you want to support me (for
whatever reason), you can message me through Github issues or via email.
[github@draganczuk.tk](mailto:github@draganczuk.tk)

I _might_ add one of those "fork me on github" thingies in the corner, though I
doubt I will

# Screenshot
![Screenshot](./screenshot.png)

# Usage
Clone this repository
```
git clone https://github.com/draganczukp/simply-shorten
```
## Building from source
Gradle 6.x.x and JDK 11 are required. Other versions are not tested
### 1. Build the `.jar` file
```
gradle build --no-daemon
```
The `--no-daemon` option means that gradle should exit as soon as the build is
finished. Without it, gradle would still be running in the background
in order to speed up future builds.

### 2. Set environment variables
```bash
# Required for authentication
export username=<api username>
export password=<api password>
# Sets where the database exists. Can be local or remote (optional)
export db_url=<url> # Default: './urls.sqlite'
```

### 3. Run it
```
java -jar build/libs/url.jar
```
You can optionally set the port the server listens on by appending `--port=[port]`

### 4. Navigate to `http://localhost:4567` in your browser, add links as you wish.

## Running with docker
### `docker run` method
0. (Only if you really want to) Build the image
```
docker build . -t simply-shorten:latest
```
1. Run the image
```
docker run -p 4567:4567
    -d url:latest
    -e username="username"
    -e password="password"
    -d simply-shorten:latest
```
1.a Make the database file available to host (optional)
```
touch ./urls.sqlite
docker run -p 4567:4567 \
    -e username="username" \
    -e password="password" \
    -v ./urls.sqlite:/urls.sqlite \
    -e db_url=/urls.sqlite \
    -d simply-shorten:latest
```
## `docker-compose`
There is a sample `docker-compose.yml` file in this repository. It contains
everything needed for a basic install. You can use it as a base, modifying
it as needed. Run it with
```
docker-compose up -d --build
```
