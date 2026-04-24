//! WASM-level integration tests for markdown-to-leaflet-wasm.
//!
//! Run with: wasm-pack test --node

use wasm_bindgen_test::*;
use markdown_to_leaflet_wasm::convert_markdown;

/// Call convert_markdown("# Hello") and assert the result is a defined JS object.
/// If std::sync::Once panics during schema initialization (the __rust_abort bug),
/// this test fails before the crate ever reaches npm.
#[wasm_bindgen_test]
fn test_convert_markdown_hello() {
    let result = convert_markdown("# Hello");
    assert!(
        result.is_ok(),
        "convert_markdown should not panic: {:?}",
        result.as_ref().unwrap_err()
    );

    let value = result.unwrap();
    assert!(!value.is_null(), "result should be a defined JS object, not null");
    assert!(
        !value.is_undefined(),
        "result should be a defined JS object, not undefined"
    );
}

/// Call convert_markdown twice in the same WASM instance.
/// std::sync::Once in no_threads mode is fatal-on-panic;
/// this ensures retry and double-init are safe.
#[wasm_bindgen_test]
fn test_idempotent_init() {
    let r1 = convert_markdown("# First");
    assert!(r1.is_ok(), "first call should succeed");
    assert!(
        !r1.as_ref().unwrap().is_null(),
        "first result should be a defined JS object"
    );

    let r2 = convert_markdown("# Second");
    assert!(
        r2.is_ok(),
        "second call in same instance should also succeed"
    );
    let value = r2.unwrap();
    assert!(!value.is_null(), "second result should be a defined JS object");
}

/// If the schema is loaded via include_str! + serde_json::from_str, test that
/// parsing in isolation succeeds with minimal input. This catches the most
/// likely root cause: an empty or missing schema file at wasm-pack time causing
/// a panic inside the Once closure.
#[wasm_bindgen_test]
fn test_schema_parse_standalone() {
    let result = convert_markdown("");
    assert!(
        result.is_ok(),
        "empty markdown should produce a valid document, not panic inside LazyLock init: {:?}",
        result.as_ref().unwrap_err()
    );
    let value = result.unwrap();
    assert!(!value.is_null(), "empty markdown should still return a defined JS object");
    assert!(
        !value.is_undefined(),
        "empty markdown should not return undefined"
    );
}
