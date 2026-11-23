const DAY_NAMES: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const SECONDS_PER_DAY: i64 = 86_400;

struct Civil {
    year: i64,
    month: usize,
    day: i64,
}

fn year_of_era(day_of_era: i64) -> i64 {
    let leaps = day_of_era / 1460 - day_of_era / 36_524 + day_of_era / 146_096;
    (day_of_era - leaps) / 365
}

fn civil_from_days(days: i64) -> Civil {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year = year_of_era(day_of_era);
    let day_of_year = day_of_era - (365 * year + year / 4 - year / 100);
    civil(year + era * 400, (5 * day_of_year + 2) / 153, day_of_year)
}

fn civil(year_base: i64, month_index: i64, day_of_year: i64) -> Civil {
    let month = (month_index + 2) % 12 + 1;
    Civil {
        year: year_base + i64::from(month <= 2),
        month: usize::try_from(month - 1).unwrap_or(0),
        day: day_of_year - (153 * month_index + 2) / 5 + 1,
    }
}

fn name_at(names: &[&'static str], index: i64) -> &'static str {
    usize::try_from(index)
        .ok()
        .and_then(|position| names.get(position))
        .copied()
        .unwrap_or_default()
}

fn calendar(civil: &Civil) -> String {
    let month = MONTH_NAMES.get(civil.month).copied().unwrap_or_default();
    format!("{:02} {month} {:04}", civil.day, civil.year)
}

fn clock(seconds: i64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
}

#[must_use]
pub fn http_date(seconds_since_epoch: i64) -> String {
    let days = seconds_since_epoch.div_euclid(SECONDS_PER_DAY);
    let seconds = seconds_since_epoch.rem_euclid(SECONDS_PER_DAY);
    let weekday = name_at(&DAY_NAMES, days.rem_euclid(7));
    format!(
        "{weekday}, {} {} GMT",
        calendar(&civil_from_days(days)),
        clock(seconds)
    )
}
