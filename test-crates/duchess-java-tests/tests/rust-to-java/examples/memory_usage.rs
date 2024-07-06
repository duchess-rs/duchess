//@run

fn main() {
    use duchess::prelude::*;
    use java::lang::management::ManagementFactory;
    let memory_bean = ManagementFactory::get_memory_mx_bean();
    let heap_usage = memory_bean
        .get_heap_memory_usage()
        .get_used()
        .execute()
        .expect("failed to load usage");
    let other_usage = memory_bean
        .get_non_heap_memory_usage()
        .get_used()
        .execute()
        .expect("failed to load usage");
    println!("total memory usage: {}", heap_usage + other_usage);
}
