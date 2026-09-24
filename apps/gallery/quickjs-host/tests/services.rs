use std::{
    sync::{Arc, Barrier, mpsc},
    time::Duration,
};

use argui_gallery_quickjs::{QuickJsGallery, ServiceOutcome, ServiceRegistry};
use serde_json::json;

#[test]
fn custom_service_routes_typed_result_to_the_calling_session() {
    let services = Arc::new(ServiceRegistry::new());
    services.register("example", "double", |payload| {
        ServiceOutcome::Ok(json!(payload["value"].as_i64().unwrap_or(0) * 2))
    });
    let (sender, responses) = mpsc::channel();
    services.submit(7, &json!({"requestId": 4, "window": "main", "service": "example", "method": "double", "payload": {"value": 9}}).to_string(), sender).unwrap();
    let response = responses.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(response.session, 7);
    assert_eq!(
        response.json(),
        json!({"requestId": 4, "window": "main", "status": "ok", "value": 18})
    );
}

#[test]
fn unsupported_service_is_reported_without_rejecting_the_protocol() {
    let services = Arc::new(ServiceRegistry::new());
    let (sender, responses) = mpsc::channel();
    assert!(!services.supports("menus", "set"));
    assert!(!services.supports("shortcuts", "set"));
    services
        .submit(
            1,
            &json!({"requestId": 1, "window": "main", "service": "menus", "method": "set"})
                .to_string(),
            sender.clone(),
        )
        .unwrap();
    let response = responses.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(response.json()["status"], "unsupported");
    assert!(
        response.json()["message"]
            .as_str()
            .unwrap()
            .contains("tray menus")
    );
    services
        .submit(
            1,
            &json!({"requestId": 2, "window": "main", "service": "shortcuts", "method": "set"})
                .to_string(),
            sender,
        )
        .unwrap();
    let response = responses.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(response.json()["status"], "unsupported");
    assert!(
        response.json()["message"]
            .as_str()
            .unwrap()
            .contains("global-shortcuts feature")
    );
}

#[test]
fn cancelled_session_does_not_receive_a_late_result() {
    let services = Arc::new(ServiceRegistry::new());
    let barrier = Arc::new(Barrier::new(2));
    let worker_barrier = Arc::clone(&barrier);
    services.register("slow", "work", move |_| {
        worker_barrier.wait();
        worker_barrier.wait();
        ServiceOutcome::Ok(json!(true))
    });
    let (sender, responses) = mpsc::channel();
    services
        .submit(
            11,
            &json!({"requestId": 3, "window": "main", "service": "slow", "method": "work"})
                .to_string(),
            sender,
        )
        .unwrap();
    barrier.wait();
    services.cancel_session(11);
    barrier.wait();
    assert!(responses.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn quickjs_receives_native_service_response_without_a_tree_transaction() {
    let (request_sender, requests) = mpsc::channel();
    let (commit_sender, commits) = mpsc::channel();
    let gallery = QuickJsGallery::new_with_services(
        r#"export function mountGallery(bridge) {
            const stop = bridge.subscribeResponses(response => {
                bridge.commit([{ kind: 'setProperty', id: { slot: 1, generation: 1 }, property: 2,
                    value: { type: 'String', value: `${response.window}:${response.value}` } }])
            })
            bridge.request({ requestId: 4, window: 'main', service: 'example', method: 'echo', payload: 'hello' })
            return () => stop()
        }"#,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        move |batch| { commit_sender.send(batch).unwrap(); String::new() },
        |_| String::new(),
        move |request| { request_sender.send(request).unwrap(); String::new() },
        |_| String::new(),
    ).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests.recv().unwrap()).unwrap()["payload"],
        "hello"
    );
    assert!(commits.try_recv().is_err());
    gallery
        .deliver_service(
            &json!({"requestId": 4, "window": "main", "status": "ok", "value": "world"})
                .to_string(),
        )
        .unwrap();
    let committed: serde_json::Value = serde_json::from_str(&commits.recv().unwrap()).unwrap();
    assert_eq!(committed[0][5], "main:world");
    gallery.dispose().unwrap();
}

#[test]
fn quickjs_service_click_supports_abort_signals_and_keeps_the_session_alive() {
    let (request_sender, requests) = mpsc::channel();
    let (cancel_sender, cancellations) = mpsc::channel();
    let gallery = QuickJsGallery::new_with_services(
        r#"export function mountGallery(bridge) {
            let controller
            bridge.subscribe(() => {
                controller = new AbortController()
                controller.signal.addEventListener('abort', () => bridge.cancelRequest('main', 1), { once: true })
                bridge.request({ requestId: 1, window: 'main', service: 'clipboard', method: 'writeText', payload: { text: 'copied' } })
            })
            bridge.subscribeResponses(response => {
                if (response.requestId === 1 && response.status === 'ok') controller.abort()
            })
            return () => {}
        }"#,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        |_| String::new(),
        |_| String::new(),
        move |request| { request_sender.send(request).unwrap(); String::new() },
        move |cancel| { cancel_sender.send(cancel).unwrap(); String::new() },
    ).unwrap();
    gallery.deliver("{}").unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests.recv().unwrap()).unwrap()["payload"]["text"],
        "copied"
    );
    gallery
        .deliver_service(r#"{"requestId":1,"window":"main","status":"ok"}"#)
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&cancellations.recv().unwrap()).unwrap(),
        json!({"window": "main", "requestId": 1})
    );
    gallery.deliver("{}").unwrap();
}

#[test]
fn service_bridge_bounds_custom_payloads_and_results() {
    let services = Arc::new(ServiceRegistry::new());
    services.register("large", "reply", |_| {
        ServiceOutcome::Ok(json!("x".repeat(70_000)))
    });
    let (sender, responses) = mpsc::channel();
    let oversized = json!({"requestId": 1, "window": "main", "service": "large", "method": "reply", "payload": "x".repeat(70_000)}).to_string();
    assert!(
        services
            .submit(1, &oversized, sender.clone())
            .unwrap_err()
            .contains("64 KiB")
    );
    services
        .submit(
            1,
            &json!({"requestId": 2, "window": "main", "service": "large", "method": "reply"})
                .to_string(),
            sender,
        )
        .unwrap();
    let response = responses.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(response.json()["status"], "error");
    assert!(
        response.json()["message"]
            .as_str()
            .unwrap()
            .contains("64 KiB")
    );
}
