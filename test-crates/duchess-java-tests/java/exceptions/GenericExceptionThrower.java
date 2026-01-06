package exceptions;

public class GenericExceptionThrower<E extends java.lang.Exception> {
    private E e_;
    public GenericExceptionThrower(E e) {
        e_ = e;
    }

    public void throwsGenericAndSimple() throws E, RuntimeException {
        throw e_;
    }
    
    public static <T, E2 extends java.lang.Exception> void throwsGenericException(T _t, E2 e) throws E2 {
        throw e;
    }

    public static RuntimeException returnsException() {
        return new RuntimeException("test exception");
    }
}
