package auth;

// XX: Mangled name only needed until we can parse innner classes
public class AuthenticationExceptionUnauthenticated extends AuthenticationException {

    public String userMessage() {
        return "not authenticated";
    }
    
}
