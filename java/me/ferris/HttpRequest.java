package me.ferris;

import java.util.List;
import java.util.Map;

public record HttpRequest(
    String verb,
    String path,
    byte[] hashedPayload,
    Map<String, List<String>> parameters
    // Map<String, ? extends List<String>> headers
) { }
