duchess::java_package! {
    package flags;

    public class flags.Flags { //~ ERROR: member declared as `private`
        private int privateField;
        public flags.Flags();
        private int privateMethod();
        private int publicMethod();
    }
}

fn main() {}
