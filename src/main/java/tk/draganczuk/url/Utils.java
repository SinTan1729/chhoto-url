package tk.draganczuk.url;

import java.util.Random;
import java.util.regex.Pattern;

public class Utils {
	private static final Random random = new Random(System.currentTimeMillis());

	private static final String SHORT_URL_PATTERN = "[a-z0-9_-]+";
	private static final Pattern PATTERN = Pattern.compile(SHORT_URL_PATTERN);

	public static String randomString() {
		int leftLimit = 48; // numeral '0'
		int rightLimit = 122; // letter 'z'
		int targetStringLength = 10;

		return random.ints(leftLimit, rightLimit + 1)
				.filter(i -> (i <= 57 || i >= 97))
				.limit(targetStringLength)
				.collect(StringBuilder::new,
						StringBuilder::appendCodePoint,
						StringBuilder::append)
				.toString();
	}

	public static boolean validate(String shortUrl) {
		return PATTERN.matcher(shortUrl)
				.matches();
	}
}
