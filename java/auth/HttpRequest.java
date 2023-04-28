
package auth;

import java.util.List;
import java.util.Map;

public record HttpRequest(
    String verb,
    String path,
    byte[] hashedPayload,
    Map<String, List<String>> parameters,
    Map<String, List<String>> headers
) { }
