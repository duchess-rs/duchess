//@compile-flags: --crate-type lib

duchess::java_package! {
    package type_mismatch;

    public class TakesInt {
        //~^ ERROR: method `take(long)` does not match any of the methods in the reflected class
        public void take(short);
    }
}

fn main() {}
