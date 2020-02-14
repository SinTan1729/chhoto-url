package tk.draganczuk.url;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.StandardOpenOption;
import java.util.List;
import java.util.Optional;


public class UrlFile {

	private File file;

	public UrlFile() throws IOException{
		String path = System.getenv().getOrDefault("file.location", "./urls.csv");
		this.file = new File(path);
		if (!file.exists()) {
			file.createNewFile();
		}
	}

	public List<String> getAll() throws IOException{
		return Files.readAllLines(file.toPath());
	}

	public String addUrl(String longURL, String shortUrl){
		String entry = String.format("%s,%s",shortUrl,longURL);
		try {
			var lineOpt = Files.lines(file.toPath())
				.filter(line -> line.equals(entry))
				.findAny();
			if(lineOpt.isEmpty()){
				Files.writeString(file.toPath(), entry + System.lineSeparator(), StandardOpenOption.APPEND);
			}
		} catch (IOException e) {
			e.printStackTrace();
		}
		return entry;
	}

	public Optional<String> findForShortUrl(String shortUrl){
		try {
			return  Files.lines(file.toPath())
				.map(this::splitLine)
				.filter(pair -> pair.getLeft().equals(shortUrl))
				.map(Pair::getRight)
				.findAny();
		} catch (IOException e) {
			return Optional.empty();
		}
	}

	public Pair<String, String> splitLine(String line){
		var split = line.split(",");
		return new Pair<>(split[0], split[1]);
	}
}
