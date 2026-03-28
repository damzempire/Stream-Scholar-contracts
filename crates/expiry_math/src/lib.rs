#![no_std]

pub const MONTH_IN_SECONDS: u64 = 30 * 24 * 60 * 60;

pub fn checked_add_time(base_time: u64, seconds_to_add: u64) -> Option<u64> {
    base_time.checked_add(seconds_to_add)
}

pub fn checked_add_minutes_to_timestamp(base_time: u64, minutes_to_add: u64) -> Option<u64> {
    minutes_to_add
        .checked_mul(60)
        .and_then(|seconds_to_add| checked_add_time(base_time, seconds_to_add))
}

pub fn checked_access_expiry(
    current_time: u64,
    existing_expiry: u64,
    seconds_bought: u64,
) -> Option<u64> {
    let base_time = if existing_expiry > current_time {
        existing_expiry
    } else {
        current_time
    };

    checked_add_time(base_time, seconds_bought)
}

pub fn checked_subscription_expiry(current_time: u64, duration_months: u64) -> Option<u64> {
    duration_months
        .checked_mul(MONTH_IN_SECONDS)
        .and_then(|duration_seconds| checked_add_time(current_time, duration_seconds))
}

#[cfg(test)]
mod tests {
    use super::{
        checked_access_expiry, checked_add_minutes_to_timestamp, checked_subscription_expiry,
    };

    #[test]
    fn returns_none_when_access_expiry_overflows() {
        assert_eq!(checked_access_expiry(u64::MAX - 5, u64::MAX - 1, 10), None);
    }

    #[test]
    fn returns_none_when_minutes_overflow() {
        assert_eq!(checked_add_minutes_to_timestamp(u64::MAX - 30, 1), None);
    }

    #[test]
    fn returns_none_when_subscription_overflows() {
        assert_eq!(checked_subscription_expiry(u64::MAX - 10, 1), None);
    }
}
