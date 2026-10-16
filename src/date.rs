//! Minimal calendar arithmetic. The real Oregon Trail cared a great deal
//! about what month you were in, so we track real dates (1848 style).

use std::fmt;

pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Date {
    pub year: i32,
    /// 1-12
    pub month: u32,
    /// 1-31
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Date { year, month, day }
    }

    pub fn is_leap(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    pub fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Date::is_leap(year) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    pub fn month_name(&self) -> &'static str {
        MONTH_NAMES[(self.month.clamp(1, 12) - 1) as usize]
    }

    /// Step forward one day.
    pub fn next_day(&mut self) {
        self.day += 1;
        if self.day > Date::days_in_month(self.year, self.month) {
            self.day = 1;
            self.month += 1;
            if self.month > 12 {
                self.month = 1;
                self.year += 1;
            }
        }
    }

    /// Days since Jan 1 of `self.year`, 0-based.
    pub fn day_of_year(&self) -> u32 {
        let mut n = self.day - 1;
        for m in 1..self.month {
            n += Date::days_in_month(self.year, m);
        }
        n
    }

    /// "the 23rd day of July, 1848"
    pub fn long_form(&self) -> String {
        format!(
            "the {}{} of {}, {}",
            self.day,
            ordinal_suffix(self.day),
            self.month_name(),
            self.year
        )
    }
}

pub fn ordinal_suffix(n: u32) -> &'static str {
    match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}, {}", self.month_name(), self.day, self.year)
    }
}
