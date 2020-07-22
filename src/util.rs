use chrono::NaiveDateTime as DateTime;

pub fn convert_time(seconds: i64) -> String {
    return DateTime::from_timestamp(seconds, 0).format("%0I:%M %P").to_string();
}