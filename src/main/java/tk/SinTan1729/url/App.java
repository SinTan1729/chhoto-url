package tk.SinTan1729.url;

import static spark.Spark.*;

public class App {

	public static void main(String[] args) {
		// Useful for developing the frontend
		// http://sparkjava.com/documentation#examples-and-faq -> How do I enable automatic refresh of static files?
		if (System.getenv("dev") != null) {
			String projectDir = System.getProperty("user.dir");
			String staticDir = "/src/main/resources/public";
			staticFiles.externalLocation(projectDir + staticDir);
		} else {
			staticFiles.location("/public");
		}

		port(Integer.parseInt(System.getenv().getOrDefault("port", "4567")));

		// Add GZIP compression
		after(Filters::addGZIP);

		// No need to auth in dev
		if (System.getenv("dev") == null && Utils.isPasswordEnabled()) {
			// Authenticate
			before("/api/*", Filters.createAuthFilter());
		}

		get("/", (req, res) -> {
			res.redirect("/index.html");
			return "Redirect";
		});


		path("/api", () -> {
			get("/all", Routes::getAll);
			post("/new", Routes::addUrl);
			delete("/:shortUrl", Routes::delete);
			get("/site", Routes::getSiteUrl);
		});

		get("/:shortUrl", Routes::goToLongUrl);
	}
}
