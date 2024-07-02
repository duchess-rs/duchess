duchess::java_package! {
    package flags;

    public class flags.Flags { //~ ERROR: member declared as `public`
        private int privateField;
        public flags.Flags();
        public int privateMethod();
        public int publicMethod();
    }
}

fn main() {}
