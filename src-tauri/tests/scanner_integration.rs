use domain_scanner_app_lib::models::task::ScanMode;
use domain_scanner_app_lib::scanner::list_generator::{estimate_pattern_count, ListGenerator};

#[test]
fn million_scale_regex_generator_can_seek_to_high_offsets() {
    let mut generator = ListGenerator::new(
        ScanMode::Regex {
            pattern: "^[a-z]{5}$".to_string(),
        },
        vec![".com".to_string(), ".net".to_string(), ".org".to_string()],
    )
    .with_batch_size(5)
    .with_start_index(3_000_000);

    assert_eq!(estimate_pattern_count("^[a-z]{5}$"), 11_881_376);
    assert_eq!(generator.total_count(), 35_644_128);
    assert_eq!(generator.current_index(), 3_000_000);

    let batch = generator.next_batch();
    assert_eq!(batch.len(), 5);
    assert_eq!(batch[0].index, 3_000_000);
    assert_eq!(batch[4].index, 3_000_004);
    assert!(batch.iter().all(|candidate| {
        candidate.domain.ends_with(".com")
            || candidate.domain.ends_with(".net")
            || candidate.domain.ends_with(".org")
    }));
}

#[test]
fn large_wildcard_space_only_returns_requested_window() {
    let mut generator = ListGenerator::new(
        ScanMode::Wildcard {
            pattern: "?????".to_string(),
        },
        vec![".com".to_string()],
    )
    .with_batch_size(10)
    .with_start_index(1_000_000);

    let batch = generator.next_batch();
    assert_eq!(batch.len(), 10);
    assert_eq!(batch[0].index, 1_000_000);
    assert_eq!(generator.current_index(), 1_000_010);
    assert!(batch
        .iter()
        .all(|candidate| candidate.domain.ends_with(".com")));
    assert!(batch.iter().all(|candidate| candidate.domain.len() == 9));
}
