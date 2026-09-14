use argui_example_ai_harness::fake_model::FakeStream;
use std::time::Duration;

#[test]
fn stream_emits_exactly_the_tokens_due_at_target_rate() {
    let mut stream = FakeStream::new(1_000, 6_000, "hello");

    assert_eq!(stream.take_due(Duration::from_millis(16)).tokens, 16);
    assert_eq!(stream.take_due(Duration::from_millis(33)).tokens, 17);
    assert_eq!(stream.emitted(), 33);
}

#[test]
fn stream_finishes_without_exceeding_its_context() {
    let mut stream = FakeStream::new(1_000, 50, "hello");
    let batch = stream.take_due(Duration::from_secs(5));

    assert_eq!(batch.tokens, 50);
    assert_eq!(stream.emitted(), 50);
    assert!(stream.is_finished());
    assert_eq!(stream.take_due(Duration::from_secs(6)).tokens, 0);
}

#[test]
fn stream_is_deterministic_for_a_prompt() {
    let mut first = FakeStream::new(1_000, 100, "same prompt");
    let mut second = FakeStream::new(1_000, 100, "same prompt");

    assert_eq!(
        first.take_due(Duration::from_secs(1)),
        second.take_due(Duration::from_secs(1))
    );
}
