//@check-pass

package java_passing_null;

public class JavaPassingNull {
    native String identity(String name);

    public static void main(String[] args) {
        System.loadLibrary("native_fn_passing_null");
        JavaPassingNull p = new JavaPassingNull();

        String in = "duchess";
        String out = p.identity("duchess");
        if (in != out)
            throw new RuntimeException("Did not get expected string");

        if (p.identity(null) != null)
            throw new RuntimeException("Did not get null");
    }
}
