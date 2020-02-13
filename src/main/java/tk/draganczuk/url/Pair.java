package tk.draganczuk.url;

/**
* Pair
*/
public class Pair<T, U> {

	private T left;
	private U right;


	public Pair(T left, U right) {
		this.left = left;
		this.right = right;
	}

	public static <T, U> Pair<T, U> of(T left, U right){
		return new Pair<T,U>(left, right);
	}

	public T getLeft() {
		return left;
	}

	public void setLeft(T left) {
		this.left = left;
	}

	public U getRight() {
		return right;
	}

	public void setRight(U right) {
		this.right = right;
	}
}
