//@run
use duchess::{java, prelude::*};

duchess::java_package! {
    package generics;

    public final class generics.EventKey implements generics.MapKey {
        public static final generics.EventKey A;
        public static final generics.EventKey B;
    }

    interface generics.MapKey { * }


    public class generics.MapLike<TT extends generics.MapKey> {
        public generics.MapLike();
        public void add(TT, java.lang.Object);
        public <T extends generics.MapKey> void methodGeneric(T, java.lang.Object);
    }
}

fn main() {
    use generics::*;
    let base = EventKey::get_a();
    let event = MapLike::<EventKey>::new().execute().unwrap();
    event.add(base, "value-a").execute().unwrap();
    event
        .method_generic::<EventKey>(EventKey::get_b(), "value-b")
        .execute()
        .unwrap();
    assert_eq!(
        event.to_string().execute().unwrap(),
        Some("A=value-a\nB=value-b\n".to_string())
    );
}
