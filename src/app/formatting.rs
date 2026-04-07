//! Number formatting and address parsing utilities.

/// Format a raw balance (in smallest unit) as a human-readable string.
pub fn format_balance(raw: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let whole = raw / divisor;
    let frac = raw % divisor;
    if decimals == 0 {
        return whole.to_string();
    }
    let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
    let trimmed = frac_str.trim_end_matches('0');
    let min_width = 2.min(frac_str.len());
    let display_frac = if trimmed.len() < min_width {
        &frac_str[..min_width]
    } else {
        trimmed
    };
    format!("{}.{}", whole, display_frac)
}

pub fn parse_address(hex: &str) -> Option<iota_sdk::types::Address> {
    iota_sdk::types::Address::from_hex(hex).ok()
}

/// Parse an IOTA amount string (decimal IOTA or raw nanos) into nanos.
pub fn parse_iota_amount(s: &str) -> Option<u64> {
    if let Ok(f) = s.parse::<f64>() {
        Some((f * 1_000_000_000.0) as u64)
    } else {
        s.parse::<u64>().ok()
    }
}

/// Format nanos as a human-readable IOTA amount.
pub fn format_iota(nanos: u128) -> String {
    let whole = nanos / 1_000_000_000;
    let frac = nanos % 1_000_000_000;
    if frac == 0 {
        format!("{}", whole)
    } else {
        let frac_str = format!("{:09}", frac);
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{}", whole, trimmed)
    }
}
