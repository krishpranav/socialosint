use socialosint::core::Core;

pub fn create_test_core() -> Core {
    Core::new(None).expect("Failed to create test Core")
}

pub fn create_test_core_with_proxy(proxy: &str) -> Core {
    Core::new(Some(proxy.to_string())).expect("Failed to create test Core with proxy")
}
