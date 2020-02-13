package tk.draganczuk.url;

import static spark.Spark.*;

public class App {

    public static void main(String[] args) {
		get("/all", Routes::getAll);
		post("/new", Routes::addUrl);
		get("/:shortUrl", Routes::goToLongUrl);
    }
}
