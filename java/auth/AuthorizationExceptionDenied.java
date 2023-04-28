package auth;

public final class AuthorizationExceptionDenied extends AuthorizationException {
    public String userMessage() {
        return "User is not allowed to do that";
    }
}
