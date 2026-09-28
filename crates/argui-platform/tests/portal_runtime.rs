#![cfg(target_os = "linux")]
#[path = "../src/portal_runtime.rs"]
mod portal_runtime;
use portal_runtime::runtime;

#[test]
fn portal_tasks_have_an_owned_executor_and_timer() {
    let first = runtime().unwrap();
    assert!(std::ptr::eq(first, runtime().unwrap()));
    assert_eq!(
        first.block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            tokio::spawn(async { 42 }).await.unwrap()
        }),
        42
    );
}
