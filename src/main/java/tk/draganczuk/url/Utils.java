package tk.draganczuk.url;

import java.util.Random;

public class Utils {
	private static final Random random = new Random(System.currentTimeMillis());

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
}
