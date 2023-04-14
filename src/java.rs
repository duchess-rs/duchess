pub mod lang;

// XX this isn't a real class in the JVM, since each array type (e.g. Foo[] and int[]) is just a subclass of Object.
// Should it go somewhere outside of the JDK core classes?
pub use crate::array::JavaArray as Array;

pub mod util {
    // pub use crate::collections::list::ArrayList;
    pub use crate::collections::list::List;
    pub use crate::collections::list::ListExt;
    pub use crate::collections::map::HashMap;
    pub use crate::collections::map::Map;
    pub use crate::collections::map::MapExt;
}
