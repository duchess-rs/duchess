package native_greeting;

public class Native {
    public String greet(String name) {
        return baseGreeting(name) + ", from Java";
    }

    native String baseGreeting(String name);
}
