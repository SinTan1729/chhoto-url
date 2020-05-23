package tk.draganczuk.url;

import org.eclipse.jetty.http.HttpStatus;
import spark.Request;
import spark.Response;

public class Routes {

	private static UrlRepository urlRepository;

	static {
		urlRepository = new UrlRepository();
	}

	public static String getAll(Request req, Response res) {
		return String.join("\n", urlRepository.getAll());
	}

	public static String addUrl(Request req, Response res) {
		String longUrl = req.queryParams("long");
		String shortUrl = req.queryParams("short");

		if (shortUrl == null || shortUrl.isBlank()) {
			shortUrl = Utils.randomString();
		}

		if (Utils.validate(shortUrl)) {
			return urlRepository.addUrl(longUrl, shortUrl);
		} else {
			res.status(HttpStatus.BAD_REQUEST_400);
			return "shortUrl not valid ([a-z0-9]+)";
		}
	}


	public static String goToLongUrl(Request req, Response res) {
		String shortUrl = req.params("shortUrl");
		var longUrlOpt = urlRepository
				.findForShortUrl(shortUrl);

		if (longUrlOpt.isEmpty()) {
			res.status(404);
			return "";
		}

		res.redirect(longUrlOpt.get());

		return "";
	}

	public static String delete(Request req, Response res) {
		String shortUrl = req.params("shortUrl");

		urlRepository.deleteEntry(shortUrl);
		return "";
	}
}
