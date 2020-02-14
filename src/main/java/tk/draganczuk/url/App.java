package tk.draganczuk.url;

import static spark.Spark.*;

public class App {

	public static void main(String[] args) {
		// Useful for developing the frontend
		// http://sparkjava.com/documentation#examples-and-faq -> How do I enable automatic refresh of static files?
		if (System.getenv("dev").equals("true")) {
			String projectDir = System.getProperty("user.dir");
			String staticDir = "/src/main/resources/public";
			staticFiles.externalLocation(projectDir + staticDir);
		} else {
			staticFiles.location("/public");
		}

		get("/", (req, res) -> {
			res.redirect("/index.html");
			return "Redirect";
		});

		get("/all", Routes::getAll);
		post("/new", Routes::addUrl);
		get("/:shortUrl", Routes::goToLongUrl);
	}
}
