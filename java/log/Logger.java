package log;

public class Logger {
    public Logger() {
        System.out.println("new Logger");
    }

    public void addEvent(Event e) {
        System.out.println("LOG: " + e.name() + " at " + e.eventTime());
    }
}
