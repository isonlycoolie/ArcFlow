use arcflow_core::retry::BackoffStrategy;

#[test]
fn exponential_backoff_grows() {
    let s = BackoffStrategy::Exponential {
        base_ms: 100,
        multiplier_x100: 200,
        max_ms: 10_000,
        jitter_ms: 0,
    };
    assert_eq!(s.compute_delay_ms(1, 0), 100);
    assert_eq!(s.compute_delay_ms(2, 0), 200);
    assert_eq!(s.compute_delay_ms(3, 0), 400);
}
