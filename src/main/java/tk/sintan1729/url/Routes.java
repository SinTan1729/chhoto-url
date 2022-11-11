package tk.sintan1729.url;

import org.eclipse.jetty.http.HttpStatus;
import spark.Request;
import spark.Response;

public class Routes {

	private static final UrlRepository urlRepository;

	static {
		urlRepository = new UrlRepository();
	}

	public static String getAll(Request req, Response res) {
		return String.join("\n", urlRepository.getAll());
	}

	public static String addUrl(Request req, Response res) {
		var body = req.body();
		var split = body.split(";");
		String longUrl = split[0];
		boolean unique = false;

		String shortUrl;
		try {
			shortUrl = split[1];
			shortUrl = shortUrl.toLowerCase();		
			if (urlRepository.findForShortUrl(shortUrl).isEmpty()) {
				unique = true;
			}
		} catch (ArrayIndexOutOfBoundsException e) {
			do {
				shortUrl = Utils.randomName();
				if (urlRepository.findForShortUrl(shortUrl).isEmpty()) {
				unique = true;
				}
			} while (unique == false);
		}

		if (unique && Utils.validate(shortUrl)) {
			return urlRepository.addUrl(longUrl, shortUrl);
		} else {
			res.status(HttpStatus.BAD_REQUEST_400);
			return "shortUrl not valid or already in use";
		}
	}

	public static String getSiteUrl(Request req, Response res) {
		return System.getenv().getOrDefault("site_url", "unset");
	}


	public static String goToLongUrl(Request req, Response res) {
		String shortUrl = req.params("shortUrl");
		shortUrl = shortUrl.toLowerCase();
		var longUrlOpt = urlRepository
				.findForShortUrl(shortUrl);

		if (longUrlOpt.isEmpty()) {
			res.status(404);
			return "";
		}
		urlRepository.addHit(shortUrl);
		res.redirect(longUrlOpt.get(), HttpStatus.PERMANENT_REDIRECT_308);

		return "";
	}

	public static String delete(Request req, Response res) {
		String shortUrl = req.params("shortUrl");

		urlRepository.deleteEntry(shortUrl);
		return "";
	}
}
