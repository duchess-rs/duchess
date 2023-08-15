package auth;

public record AuthenticateRequest(
    String requestId,
    HttpRequest request,
    String action
) { }
