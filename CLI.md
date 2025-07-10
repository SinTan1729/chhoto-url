## Official CLI App
There's an official CLI app for Linux. It's maintained by me, even though I cannot promise to provide proper support. Take a look at it
[here](https://github.com/SinTan1729/chhoto-url-cli). The instructions below describe how to use all the features using `curl`.

## Instructions for CLI usage
The application can be used from the terminal using something like `curl`. In all the examples
below, replace `http://localhost:4567` with where your instance of `chhoto-url` is accessible.

You can get the version of `chhoto-url` the server is running using `curl http://localhost:4567/api/version` and
get the siteurl using `curl http://localhost:4567/api/siteurl`. These routes are accessible without any authentication.

### API key validation
**This is required for programs that rely on a JSON response from Chhoto URL**

In order to use API key validation, set the `api_key` environment variable. If this is not set, the API will default to cookie
validation (see section above). If the API key is insecure, a warning will be outputted along with a generated API key which may be used.

**All responses for requests using API key are JSON encoded.**

Example Linux command for generating a secure API key: `tr -dc A-Za-z0-9 </dev/urandom | head -c 128`

To add a link:
```bash
curl -X POST -H "X-API-Key: <YOUR_API_KEY>" -d '{"shortlink":"<shortlink>", "longlink":"<longlink>", "expiry_delay": <expiry_delay>}' http://localhost:4567/api/new
```
An empty or missing `<shortlink>` will result in it being auto-generated. 
Expiry delay is in seconds. It is capped to a maximum of 5 years. A missing `<expiry_delay>` or a value of 0 will disable expiry.

The server will reply in the following format.
```json
{
    "success": true,
    "error": false,
    "shorturl": "<shortlink>",
    "expiry_time": <expiry_time>
}
```
or
```json
{
    "success": false,
    "error": true,
    "reason": "<reason>"
}
```

To get the config for the backend:
```bash
curl -H "X-API-Key: <YOUR_API_KEY>" -d '<shortlink>' http://localhost:4567/api/getconfig
```
(This would work without authentication in public mode.)  
The server will reply in the following format.
```json
{
    "version": "<version>",
    "site_url": "<site_url>",
    "allow_capital_letters": true/false,
    "public_mode": true/false,
    "public_mode_expiry_delay": "<delay>",
    "slug_style": "<style>",
    "slug_length": "<len>",
    "try_longer_slug": true/false
}
```

```json
{
    "success": false,
    "error": true,
    "reason": "<reason>"
}
```

To get information about a single shortlink:
```bash
curl -H "X-API-Key: <YOUR_API_KEY>" -d '<shortlink>' http://localhost:4567/api/expand
```
The server will reply in the following format.
```json
{
    "success": true,
    "error": false,
    "longurl": "<longurl>",
    "hits": "<hits>",
    "expiry_time": <expiry_time>
}
```
or
```json
{
    "success": false,
    "error": true,
    "reason": "<reason>"
}
```
(This route is not accessible using cookie validation.)

To get a list of all the currently available links:
```bash
curl -H "X-API-Key: <YOUR_API_KEY>" http://localhost:4567/api/all
```

To delete a link:
```bash
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
request, please add `-b cookie.txt` to provide authentication. Unless specified, all API methods should work with cookies.

## Disable authentication
If you do not define a password environment variable when starting the docker image, authentication
will be disabled.

This if not recommended in actual use however, as it will allow anyone to create new links and delete
old ones. This might not seem like a bad idea, until you have hundreds of links
pointing to illegal content. Since there are no logs, it's impossible to prove
that those links aren't created by you.
