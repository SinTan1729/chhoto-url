package tk.draganczuk.url;

import java.io.IOException;
import java.util.List;
import java.util.UUID;

import org.eclipse.jetty.util.log.Log;

import spark.Request;
import spark.Response;

public class Routes {

	private static UrlFile urlFile;

	static{
		try {
			urlFile = new UrlFile();
		} catch (IOException e) {
			e.printStackTrace();
		}
	}

	public static List<String> getAll(Request req, Response res) throws IOException{
		return urlFile.getAll();
	}

	public static String addUrl(Request req, Response res) {
		String longUrl = req.queryParams("long");
		String shortUrl = req.queryParams("short");

 		if (shortUrl == null) {
 			shortUrl = Utils.randomString();
 		}

		return urlFile.addUrl(longUrl, shortUrl);
	}

	public static String goToLongUrl(Request req, Response res){
		String shortUrl = req.params("shortUrl");
		var longUrlOpt = urlFile
			.findForShortUrl(shortUrl);

		if(longUrlOpt.isEmpty()){
			res.status(404);
			return "";
		}

		res.redirect(longUrlOpt.get());

		return "";
	}

}
