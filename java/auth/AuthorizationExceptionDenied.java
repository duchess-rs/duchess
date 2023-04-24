package auth;

public final class AuthorizationExceptionDenied extends AuthenticationException {
    public String userMessage() {
        return "User is not allowed to do that";
    }
}
