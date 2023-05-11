use duchess::prelude::*;
use duchess::java::lang::{Object, String};
use viper::viper::silicon::Silicon;
use viper::viper::carbon::CarbonVerifier;
use viper::viper::silver::reporter::NoopReporter__;
use viper::scala::Tuple2;
use viper::scala::collection::{IterableOnceOps, IterableOnceOpsExt};
use viper::scala::collection::IterableOps;
use viper::scala::collection::SeqOps;
use viper::scala::collection::StrictOptimizedSeqOps;
use viper::scala::collection::mutable::ArrayBuffer;

type DebugInfoItem = Tuple2<String, Object>;

#[test]
fn test_construct_silicon() -> duchess::GlobalResult<()> {
    duchess::Jvm::with(|jvm| {
        let _silicon = Silicon::new().execute_with(jvm)?;
        let reporter = NoopReporter__::get_module();
        let debug_info = ArrayBuffer::new()
            .upcast::<StrictOptimizedSeqOps<DebugInfoItem, ArrayBuffer, ArrayBuffer<DebugInfoItem>>>()
            .upcast::<SeqOps<DebugInfoItem, ArrayBuffer, ArrayBuffer<DebugInfoItem>>>()
            .upcast::<IterableOps<DebugInfoItem, ArrayBuffer, ArrayBuffer<DebugInfoItem>>>()
            .upcast::<IterableOnceOps<DebugInfoItem, ArrayBuffer, ArrayBuffer<DebugInfoItem>>>()
            .to_seq();
        let _carbon = CarbonVerifier::new(reporter, debug_info).execute_with(jvm)?;
        Ok(())
    })
}
