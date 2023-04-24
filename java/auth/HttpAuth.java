package auth;

public final class HttpAuth {

    public Authenticated authenticate(HttpRequest request) throws AuthenticationException {
        return new Authenticated();
    }

    public void authorize(Authenticated authenticated, AuthorizeRequest request) throws AuthorizationException {

    }

}