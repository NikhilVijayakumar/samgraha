#[macro_export]
macro_rules! code_inventory {
    () => {
        include!(concat!(env!("OUT_DIR"), "/code_inventory.rs"));
    };
}

code_inventory!();
