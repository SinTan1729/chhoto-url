package tk.draganczuk.url;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;


public class UrlRepository {
	private static final String INSERT_ROW_SQL = "INSERT INTO urls (long_url, short_url) VALUES (?, ?)";
	private static final String CREATE_TABLE_SQL = "CREATE TABLE IF NOT EXISTS urls\n" +
			"(\n" +
			"    id          INTEGER PRIMARY KEY AUTOINCREMENT,\n" +
			"    long_url    TEXT    NOT NULL,\n" +
			"    short_url   TEXT    NOT NULL\n" +
			");";
	private static final String SELECT_FOR_SHORT_SQL = "SELECT long_url FROM urls WHERE short_url = ?";
	private static final String DELETE_ROW_SQL = "DELETE FROM urls WHERE short_url = ?";

	private String databaseUrl;


	public UrlRepository() {
		String path = System.getenv().getOrDefault("db.url", "./urls.sqlite");

		databaseUrl = "jdbc:sqlite:" + path;

		try (Connection conn = DriverManager.getConnection(databaseUrl)) {
			if (conn != null) {
				DatabaseMetaData meta = conn.getMetaData();

				conn.createStatement()
						.execute(CREATE_TABLE_SQL);

				System.out.println("Database initialised");
			}

		} catch (SQLException e) {
			System.out.println(e.getMessage());
		}
	}

	public List<String> getAll() {
		try (final var con = DriverManager.getConnection(databaseUrl)) {
			var statement = con.createStatement();

			statement.execute("SELECT * FROM urls");
			ResultSet rs = statement.getResultSet();

			List<String> result = new ArrayList<>();

			while (rs.next()) {
				result.add(String.format("%s,%s", rs.getString("short_url"), rs.getString("long_url")));
			}

			return result;

		} catch (SQLException e) {
			e.printStackTrace();
		}
		return List.of();
	}

	public String addUrl(String longURL, String shortUrl) {
		try (final var con = DriverManager.getConnection(databaseUrl)) {
			final var stmt = con.prepareStatement(INSERT_ROW_SQL);
			stmt.setString(1, longURL);
			stmt.setString(2, shortUrl);
			if (stmt.execute()) {
				return String.format("%s,%s", shortUrl, longURL);
			}
		} catch (SQLException e) {
			e.printStackTrace();
		}
		return "";
	}

	public Optional<String> findForShortUrl(String shortUrl) {
		try (final var con = DriverManager.getConnection(databaseUrl)) {
			final var stmt = con.prepareStatement(SELECT_FOR_SHORT_SQL);
			stmt.setString(1, shortUrl);
			if (stmt.execute()) {
				ResultSet rs = stmt.getResultSet();
				if (rs.next()) {
					return Optional.of(rs.getString("long_url"));
				}
			}
			return Optional.empty();
		} catch (SQLException e) {
			e.printStackTrace();
		}
		return Optional.empty();
	}

	public void deleteEntry(String shortUrl) {
		try (final var con = DriverManager.getConnection(databaseUrl)) {
			final var stmt = con.prepareStatement(DELETE_ROW_SQL);
			stmt.setString(1, shortUrl);
			stmt.execute();
		} catch (SQLException e) {
			e.printStackTrace();
		}
	}
}
