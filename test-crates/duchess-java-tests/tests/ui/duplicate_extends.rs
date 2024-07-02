duchess::java_package! {
    package log;

    public class log.Builder extends java.lang.Object, java.lang.Object {} //~ ERROR: declared interface
                                     //~^ ERROR: duplicate reference
}

fn main() {}
