#![no_main]

use arbitrary::Arbitrary;
use expiry_math::{
    checked_access_expiry, checked_add_minutes_to_timestamp, checked_subscription_expiry,
};
use libfuzzer_sys::fuzz_target;

const YEAR_2038_TS: u64 = 2_145_916_800;
const YEAR_2100_TS: u64 = 4_102_444_800;
const EDGE_WINDOW_SECONDS: u64 = 31 * 24 * 60 * 60;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    anchor_selector: bool,
    offset_seconds: u64,
    existing_expiry_offset_seconds: u64,
    minutes_to_add: u64,
    seconds_bought: u64,
    duration_months: u64,
}

fn anchored_timestamp(anchor_selector: bool, offset_seconds: u64) -> u64 {
    let anchor = if anchor_selector {
        YEAR_2100_TS
    } else {
        YEAR_2038_TS
    };

    anchor.saturating_add(offset_seconds % EDGE_WINDOW_SECONDS)
}

fuzz_target!(|input: FuzzInput| {
    let current_time = anchored_timestamp(input.anchor_selector, input.offset_seconds);
    let existing_expiry = anchored_timestamp(input.anchor_selector, input.existing_expiry_offset_seconds);

    let minute_result = checked_add_minutes_to_timestamp(current_time, input.minutes_to_add);
    if let Some(expiry_time) = minute_result {
        assert!(expiry_time >= current_time);
    }

    let access_result = checked_access_expiry(current_time, existing_expiry, input.seconds_bought);
    if let Some(expiry_time) = access_result {
        let base_time = if existing_expiry > current_time {
            existing_expiry
        } else {
            current_time
        };
        assert!(expiry_time >= base_time);
    }

    let subscription_result = checked_subscription_expiry(current_time, input.duration_months);
    if let Some(expiry_time) = subscription_result {
        assert!(expiry_time >= current_time);
    }
});
