duchess::java_package! {
    package log;

    public class log.Builder implements log.BuildStep, log.BuildStep {} //~ ERROR: duplicate reference
}

fn main() {}
