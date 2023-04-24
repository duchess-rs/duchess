package auth;

import java.util.Map;

public record AuthorizeRequest(
    String resource,
    String action,
    Map<String, String> context
) { }
