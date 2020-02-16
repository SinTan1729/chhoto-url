package tk.draganczuk.url;

import org.eclipse.jetty.http.HttpStatus;
import spark.Request;
import spark.Response;

import java.io.IOException;

public class Routes {

	private static UrlFile urlFile;

	static {
		try {
			urlFile = new UrlFile();
		} catch (IOException e) {
			e.printStackTrace();
		}
	}

	public static String getAll(Request req, Response res) throws IOException {
		return String.join("\n", urlFile.getAll());
	}

	public static String addUrl(Request req, Response res) {
		String longUrl = req.queryParams("long");
		String shortUrl = req.queryParams("short");

		if (shortUrl == null || shortUrl.isBlank()) {
			shortUrl = Utils.randomString();
		}

		if (Utils.validate(shortUrl)) {
			return urlFile.addUrl(longUrl, shortUrl);
		} else {
			res.status(HttpStatus.BAD_REQUEST_400);
			return "shortUrl not valid ([a-z0-9]+)";
		}
	}


	public static String goToLongUrl(Request req, Response res) {
		String shortUrl = req.params("shortUrl");
		var longUrlOpt = urlFile
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
		var longUrlOpt = urlFile
				.findForShortUrl(shortUrl);

		if (longUrlOpt.isEmpty()) {
			res.status(404);
			return "";
		}

		urlFile.deleteEntry(String.format("%s,%s", shortUrl, longUrlOpt.get()));
		return "";
	}
}
