use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!(
        "cargo:rustc-env=BROWSERS_BUILD_TIMESTAMP={}",
        format_build_time(secs)
    );
}

/// Format a Unix timestamp as YYYY.MM.DD.hh.mm.ss in UTC.
fn format_build_time(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}.{month:02}.{day:02}.{hour:02}.{minute:02}.{second:02}",
        hour = rem / 3600,
        minute = (rem % 3600) / 60,
        second = rem % 60,
    )
}

/// Days since 1970-01-01 → (year, month, day). Howard Hinnant's
/// civil-from-days algorithm; proleptic Gregorian calendar, UTC.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_timestamps() {
        // 1970-01-01 00:00:00 UTC
        assert_eq!(format_build_time(0), "1970.01.01.00.00.00");
        // 2000-02-29 12:34:56 UTC (leap day)
        assert_eq!(format_build_time(951_827_696), "2000.02.29.12.34.56");
        // 2026-01-01 00:00:00 UTC
        assert_eq!(format_build_time(1_767_225_600), "2026.01.01.00.00.00");
    }
}
