package tk.draganczuk.url;

import com.qmetric.spark.authentication.AuthenticationDetails;
import com.qmetric.spark.authentication.BasicAuthenticationFilter;
import spark.Filter;
import spark.Request;
import spark.Response;

public class Filters {
    public static void addGZIP(Request request, Response response) {
        response.header("Content-Encoding", "gzip");
    }

    public static Filter createAuthFilter() {
        String username = System.getenv("username");
        String password = System.getenv("password");

        return new BasicAuthenticationFilter(new AuthenticationDetails(username, password));
    }
}
