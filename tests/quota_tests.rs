use chrono::{Duration, Utc};
use switchx::model::UsageEvent;
use switchx::quota::{format_duration, QuotaCalculator, QuotaHealth};

#[test]
fn test_sliding_window_normal_capacity() {
    let now = Utc::now();
    let events = vec![
        UsageEvent {
            timestamp: now - Duration::hours(1),
            tokens_or_units: 10,
        },
        UsageEvent {
            timestamp: now - Duration::hours(2),
            tokens_or_units: 15,
        },
    ];

    let (five_h, weekly, health) = QuotaCalculator::compute_status(now, 50, 500, &events);

    assert_eq!(five_h.used, 25);
    assert_eq!(five_h.remaining, 25);
    assert_eq!(five_h.percent_remaining, 50.0);
    assert_eq!(health, QuotaHealth::Moderate);
    assert_eq!(weekly.used, 25);
}

#[test]
fn test_sliding_window_expiration() {
    let now = Utc::now();
    // Event from 6 hours ago should NOT be included in the 5-hour window
    let events = vec![
        UsageEvent {
            timestamp: now - Duration::hours(6),
            tokens_or_units: 30,
        },
        UsageEvent {
            timestamp: now - Duration::hours(1),
            tokens_or_units: 10,
        },
    ];

    let (five_h, weekly, _) = QuotaCalculator::compute_status(now, 50, 500, &events);

    assert_eq!(five_h.used, 10);
    assert_eq!(five_h.remaining, 40);
    // Weekly includes events from 6 hours ago
    assert_eq!(weekly.used, 40);
}

#[test]
fn test_throttled_countdown_calculation() {
    let now = Utc::now();
    let event_time = now - Duration::hours(4);
    let events = vec![UsageEvent {
        timestamp: event_time,
        tokens_or_units: 50,
    }];

    let (five_h, _, health) = QuotaCalculator::compute_status(now, 50, 500, &events);

    assert_eq!(five_h.used, 50);
    assert_eq!(five_h.remaining, 0);
    assert_eq!(health, QuotaHealth::Throttled);
    // Should reset in 1 hour (3600 seconds)
    assert!(five_h.resets_in_seconds > 3500 && five_h.resets_in_seconds <= 3600);
}

#[test]
fn test_duration_formatting() {
    assert_eq!(format_duration(45), "45s");
    assert_eq!(format_duration(125), "2m 5s");
    assert_eq!(format_duration(3665), "1h 1m 5s");
    assert_eq!(format_duration(90061), "1d 1h 1m");
}
