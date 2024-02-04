use chrono::{Local, LocalResult, TimeZone};

pub struct Text {
    pub site_name_default: &'static str,
    pub author_default: &'static str,
    pub format_time: fn(i64) -> String, // in: timestamp, out: formatted time
}

impl Text {
    pub fn en() -> Self {
        let format_time = |timestamp: i64| {
            let dt = match Local.timestamp_opt(timestamp, 0) {
                LocalResult::Single(dt) => dt,
                _ => return format!("(invalid timestamp: {})", timestamp),
            };

            dt.format("%B %d, %Y  %H:%M").to_string()
        };

        Self {
            site_name_default: "My Blog",
            author_default: "Author",
            format_time,
        }
    }

    pub fn zh() -> Self {
        let format_time = |timestamp: i64| {
            let dt = match Local.timestamp_opt(timestamp, 0) {
                LocalResult::Single(dt) => dt,
                _ => return format!("(invalid timestamp: {})", timestamp),
            };

            dt.format("%Y 年 %m 月 %d 日  %H:%M").to_string()
        };

        Self {
            site_name_default: "我的博客",
            author_default: "作者",
            format_time,
        }
    }
}
