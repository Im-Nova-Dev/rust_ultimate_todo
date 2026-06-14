//! Relative and absolute due-date parsing for task entry.

use chrono::{Days, NaiveDate};

pub fn parse_relative_date(input: &str, today: NaiveDate) -> Option<NaiveDate> {
    let s = input.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    match s.as_str() {
        "today" => return Some(today),
        "tomorrow" => return today.checked_add_days(Days::new(1)),
        "yesterday" => return today.checked_sub_days(Days::new(1)),
        _ => {}
    }

    let positive_num_str = if let Some(rest) = s.strip_prefix('+') {
        Some(rest.trim_end_matches('d').trim_end_matches(" days").trim())
    } else {
        s.strip_prefix("in ")
            .map(|rest| rest.trim_end_matches('d').trim_end_matches(" days").trim())
    };

    if let Some(ns) = positive_num_str
        && let Ok(num) = ns.parse::<u64>()
    {
        if num == 0 {
            return Some(today);
        }
        if num > 365_000 {
            return None;
        }
        return today.checked_add_days(Days::new(num));
    }

    if let Some(rest) = s.strip_prefix('-') {
        let ns = rest.trim_end_matches('d').trim();
        if let Ok(num) = ns.parse::<u64>() {
            if num > 365_000 {
                return None;
            }
            return today.checked_sub_days(Days::new(num));
        }
    }

    if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%m/%d/%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%d-%m-%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%d/%m/%Y") {
        return Some(d);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;

    #[test]
    fn test_parse_relative_date_basic() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        assert_eq!(parse_relative_date("today", today), Some(today));
        assert_eq!(
            parse_relative_date("tomorrow", today),
            Some(today + Days::new(1))
        );
        assert_eq!(
            parse_relative_date("yesterday", today),
            Some(today - Days::new(1))
        );
        assert_eq!(parse_relative_date("+5", today), Some(today + Days::new(5)));
        assert_eq!(
            parse_relative_date("in 3 days", today),
            Some(today + Days::new(3))
        );
        assert_eq!(
            parse_relative_date("-2d", today),
            Some(today - Days::new(2))
        );
        assert_eq!(
            parse_relative_date("2026-06-20", today),
            Some(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap())
        );
        assert_eq!(
            parse_relative_date("20/06/2026", today),
            Some(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap())
        );
    }

    #[test]
    fn test_parse_relative_date_huge_number_no_crash() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        assert_eq!(parse_relative_date("+2222222222222", today), None);
        assert_eq!(parse_relative_date("in 999999 days", today), None);
        assert_eq!(parse_relative_date("-999999", today), None);
    }

    #[test]
    fn test_parse_relative_date_edge_cases() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        assert_eq!(parse_relative_date("+0", today), Some(today));
        assert_eq!(parse_relative_date("in 0 days", today), Some(today));
        assert_eq!(parse_relative_date("", today), None);
        assert_eq!(parse_relative_date("   ", today), None);
    }
}
