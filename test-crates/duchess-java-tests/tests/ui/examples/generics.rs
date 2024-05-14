//@run
use duchess::{java, prelude::*};

duchess::java_package! {
    package generics;

    public final class generics.EventKey {
        public static final generics.EventKey A;
    }

    interface generics.MapKey { * }

    public class generics.MapLike<T extends generics.MapKey> {
        public generics.MapLike();
        public void add(T, java.lang.Object);
    }
}

fn main() {
    use generics::*;
    let base = EventKey::get_a();
    let event = MapLike::<EventKey>::new().execute().unwrap();
    event.add(base, "value").execute().unwrap();
}
