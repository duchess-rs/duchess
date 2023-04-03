package me.ferris;

public final class HttpAuth {

    public HttpAuth() {

    }

    public AuthenticateResult authenticate(HttpRequest request) throws AuthenticationException {
        return new AuthenticateResult("some-account-id", "some-principal");
    }

}