package me.ferris;

class Logger {
    public Logger() {

    }

    public void logInt(int data) {
        System.out.println("logInt(" + data + ")");
    }

    public void logString(String data) {
        System.out.println("logString(" + data + ")");
    }

    public void throwSomething() {
        throw new RuntimeException("catch me if you can!");
    }
}