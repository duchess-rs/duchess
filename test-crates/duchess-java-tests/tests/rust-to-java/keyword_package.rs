//@run
use duchess::prelude::*;

// Java package segments that collide with Rust keywords (e.g. `impl`) cannot
// spell a generated Rust name, so they are escaped with a trailing `_` (see
// `Id::to_ident` in duchess-reflect).

duchess::java_package! {
    package keyword.api;

    public interface keyword.api.Provider {
        public abstract void start();
    }

    package keyword.impl;

    public class keyword.impl.ProviderImpl implements keyword.api.Provider {
        public keyword.impl.ProviderImpl();
        public void start();
        public static int count();
    }
}

pub fn main() -> duchess::Result<()> {
    let provider: duchess::Java<keyword::impl_::ProviderImpl> = keyword::impl_::ProviderImpl::new()
        .assert_not_null()
        .execute()?;

    // `start` is declared on the interface, not on `ProviderImpl`; it must still
    // resolve on the concrete handle.
    provider.start().execute()?;
    assert_eq!(keyword::impl_::ProviderImpl::count().execute()?, 1);

    Ok(())
}
