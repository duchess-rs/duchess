package exceptions;

public class ThrowExceptions {
    public static String staticStringNotNull = "notnull";
    public static String nullString;

    public void throwRuntime() {
        throw new RuntimeException("something has gone horribly wrong");
    }

    public Object nullObject() {
        Object a = null;
        return a;
    }

    public void throwExceptionWithCrashingMessage() throws MessageRetrievalException {
        throw new MessageRetrievalException();
    }

    public class MessageRetrievalException extends Exception {

        public MessageRetrievalException() {
        }

        public String getMessage() {
            // Throw an exception when attempting to retrieve the message
            throw new RuntimeException("My exception threw an exception");
        }
    }
}
