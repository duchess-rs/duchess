package me.ferris;

public record AuthenticateResult(
    String accountId,
    String principal
) { }
