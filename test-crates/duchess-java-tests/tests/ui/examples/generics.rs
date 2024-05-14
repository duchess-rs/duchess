//@run
use duchess::{java, prelude::*};

duchess::java_package! {
    package generics;

    public final class generics.EventKey {
        public static final generics.EventKey A;
    }

    interface generics.MapKey { * }


    public class generics.MapLike<TT extends generics.MapKey> {
        public generics.MapLike();
        public void add(TT, java.lang.Object);
    }
}

fn main() {
    use generics::*;
    let base = EventKey::get_a();
    let event = MapLike::<EventKey>::new().execute().unwrap();
    event.add(base, "value").execute().unwrap();
    assert_eq!(
        event.to_string().execute().unwrap(),
        Some("A=value\n".to_string())
    );
}
