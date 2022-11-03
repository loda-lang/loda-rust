use chrono::prelude::*;

pub struct LastAnalyticsTimestamp {

}

impl LastAnalyticsTimestamp {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_format() {
        let dt: DateTime<Utc> = Utc.ymd(1999, 3, 24).and_hms_micro(21, 59, 33, 453_829);
        let s = dt.to_rfc3339_opts(SecondsFormat::Secs, true).to_string();
        assert_eq!(s, "1999-03-24T21:59:33Z"); // release date of "the matrix"
    }

    #[test]
    fn test_10000_parse() {
        let date_str = "1999-03-24T21:59:33Z"; // release date of "the matrix"
        let datetime: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(date_str).unwrap();
        let dt: DateTime<Utc> = datetime.with_timezone(&Utc);
        assert_eq!(dt.year(), 1999);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 24);
        assert_eq!(dt.hour(), 21);
        assert_eq!(dt.minute(), 59);
    }
}
