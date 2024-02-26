use core::fmt::Display;
use crate::internal::cmos::{Rtc};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
} impl Month {
    pub fn from_u8(month: u8) -> Option<Self> {
        match month {
            1 => Some(Month::January),
            2 => Some(Month::February),
            3 => Some(Month::March),
            4 => Some(Month::April),
            5 => Some(Month::May),
            6 => Some(Month::June),
            7 => Some(Month::July),
            8 => Some(Month::August),
            9 => Some(Month::September),
            10 => Some(Month::October),
            11 => Some(Month::November),
            12 => Some(Month::December),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Weekday {
    Saturday = 0,
    Sunday = 1,
    Monday = 2,
    Tuesday = 3,
    Wednesday = 4,
    Thursday = 5,
    Friday = 6,
} impl Weekday {
    pub fn from_u8(weekday: u8) -> Option<Self> {
        match weekday {
            0 => Some(Weekday::Saturday),
            1 => Some(Weekday::Sunday),
            2 => Some(Weekday::Monday),
            3 => Some(Weekday::Tuesday),
            4 => Some(Weekday::Wednesday),
            5 => Some(Weekday::Thursday),
            6 => Some(Weekday::Friday),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Duration {
    nanos: u64,
    seconds: u64,
} #[allow(dead_code)] impl Duration {
    pub fn new(nanos: u64, seconds: u64) -> Self { Self {
        nanos, seconds,
    } }

    pub fn from_nanos(nanos: u64) -> Self {
        Self {
            nanos,
            seconds: 0,
        }
    }

    pub fn from_micros(micros: u64) -> Self {
        Self {
            nanos: micros * 1000,
            seconds: 0,
        }
    }

    pub fn from_millis(millis: u64) -> Self {
        Self {
            nanos: millis * 1000000,
            seconds: 0,
        }
    }

    pub fn from_seconds(seconds: u64) -> Self {
        Self {
            nanos: 0,
            seconds,
        }
    }

    pub fn from_minutes(minutes: u64) -> Self {
        Self {
            nanos: 0,
            seconds: minutes * 60,
        }
    }

    pub fn from_hours(hours: u64) -> Self {
        Self {
            nanos: 0,
            seconds: hours * 3600,
        }
    }

    pub fn from_days(days: u64) -> Self {
        Self {
            nanos: 0,
            seconds: days * 86400,
        }
    }

    pub fn nanos(&self) -> u64 { self.nanos }

    pub fn micros(&self) -> u64 { self.nanos / 1000 }

    pub fn millis(&self) -> u64 { self.nanos / 1000000 }

    pub fn seconds(&self) -> u64 { self.seconds }

    pub fn minutes(&self) -> u64 { self.seconds / 60 }

    pub fn hours(&self) -> u64 { self.seconds / 3600 }

    pub fn days(&self) -> u64 { self.seconds / 86400 }

    pub fn as_seconds(&self) -> f64 {
        self.seconds as f64 + (self.nanos as f64 / 1_000_000_000.0)
    }

    pub fn as_minutes(&self) -> f64 {
        self.minutes() as f64 + (self.seconds as f64 / 60.0)
    }

    pub fn as_hours(&self) -> f64 {
        self.hours() as f64 + (self.minutes() as f64 / 60.0)
    }

    pub fn as_days(&self) -> f64 {
        self.days() as f64 + (self.hours() as f64 / 24.0)
    }

    pub fn add(&self, rhs: Self) -> Option<Self> {
        let nanos = self.nanos.checked_add(rhs.nanos)?;
        let seconds = self.seconds.checked_add(rhs.seconds)?;
        Some(Self::new(nanos, seconds))
    }

    pub fn sub(&self, rhs: Self) -> Option<Self> {
        let nanos = self.nanos.checked_sub(rhs.nanos)?;
        let seconds = self.seconds.checked_sub(rhs.seconds)?;
        Some(Self::new(nanos, seconds))
    }

    pub fn mul(&self, rhs: u64) -> Option<Self> {
        let nanos = self.nanos.checked_mul(rhs)?;
        let seconds = self.seconds.checked_mul(rhs)?;
        Some(Self::new(nanos, seconds))
    }

    pub fn div(&self, rhs: u64) -> Option<Self> {
        let nanos = self.nanos.checked_div(rhs)?;
        let seconds = self.seconds.checked_div(rhs)?;
        Some(Self::new(nanos, seconds))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    nano: u32,
    seconds: u8,
    minutes: u8,
    hours: u8,
} #[allow(dead_code)] impl Time {
    pub fn new(nano: u32, seconds: u8, minutes: u8, hours: u8) -> Self { Self {
        nano, seconds, minutes, hours,
    } }

    pub fn nano(&self) -> u32 { self.nano }

    pub fn micro(&self) -> u32 { self.nano / 1000 }

    pub fn milli(&self) -> u32 { self.nano / 1000000 }

    pub fn seconds(&self) -> u8 { self.seconds }

    pub fn minutes(&self) -> u8 { self.minutes }

    pub fn hours(&self) -> u8 { self.hours }

    pub fn as_hms(&self) -> (u8, u8, u8) { (self.hours, self.minutes, self.seconds) }

    pub fn as_hms_milli(&self) -> (u8, u8, u8, u32) { (self.hours, self.minutes, self.seconds, self.milli()) }

    pub fn as_hms_micro(&self) -> (u8, u8, u8, u32) { (self.hours, self.minutes, self.seconds, self.micro()) }

    pub fn as_hms_nano(&self) -> (u8, u8, u8, u32) { (self.hours, self.minutes, self.seconds, self.nano()) }

    pub fn add(&self, rhs: Duration) -> Option<Self> {
        let nano = self.nano.checked_add(rhs.nanos as u32)?;
        let seconds = self.seconds.checked_add(rhs.seconds as u8)?;
        let minutes = self.minutes.checked_add(seconds / 60)?;
        let hours = self.hours.checked_add(minutes / 60)?;
        Some(Self::new(nano, seconds, minutes, hours))
    }

    pub fn sub(&self, rhs: Duration) -> Option<Self> {
        let nano = self.nano.checked_sub(rhs.nanos as u32)?;
        let seconds = self.seconds.checked_sub(rhs.seconds as u8)?;
        let minutes = self.minutes.checked_sub(seconds / 60)?;
        let hours = self.hours.checked_sub(minutes / 60)?;
        Some(Self::new(nano, seconds, minutes, hours))
    }
} impl From<Rtc> for Time {
    fn from(rtc: Rtc) -> Self {
        Self {
            nano: 0,
            seconds: rtc.seconds,
            minutes: rtc.minutes,
            hours: rtc.hours,
        }
    }
} impl Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}.{:03}", self.hours, self.minutes, self.seconds, self.milli())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Date {
    day: u8,
    month: Month,
    year: i32,
} #[allow(dead_code)] impl Date {
    pub fn new(day: u8, month: Month, year: i32) -> Self { Self {
        day, month, year,
    } }

    pub fn day(&self) -> u8 { self.day }

    pub fn ordinal(&self) -> u16 {
        let mut ordinal = self.day as u16;
        for month in 1..self.month as u8 {
            ordinal += Date::new(1, Month::from_u8(month).unwrap(), self.year).days_in_month() as u16;
        }
        ordinal
    }

    pub fn week(&self) -> u8 {
        let mut ordinal = self.ordinal();
        let mut week = 1;
        while ordinal > 7 {
            ordinal -= 7;
            week += 1;
        }
        week
    }

    pub fn weekday(&self) -> Weekday {
        let mut year = self.year;
        let mut month = self.month as u8;
        let day = self.day as i32;

        if month < 3 {
            month += 12;
            year -= 1;
        }

        let k = year % 100;
        let j = year / 100;
        let h = (day + (13 * (month as i32 + 1) / 5) + k + (k / 4) + (j / 4) + 5 * j) % 7;

        Weekday::from_u8(h as u8).unwrap()
    }

    pub fn month(&self) -> Month { self.month }

    pub fn year(&self) -> i32 { self.year }

    pub fn is_leap_year(&self) -> bool {
        (self.year % 4 == 0) && (self.year % 100 != 0 || self.year % 400 == 0)
    }

    pub fn days_in_month(&self) -> u8 {
        match self.month {
            Month::January => 31,
            Month::February => if self.is_leap_year() { 29 } else { 28 },
            Month::March => 31,
            Month::April => 30,
            Month::May => 31,
            Month::June => 30,
            Month::July => 31,
            Month::August => 31,
            Month::September => 30,
            Month::October => 31,
            Month::November => 30,
            Month::December => 31,
        }
    }

    pub fn add(&self, rhs: Duration) -> Option<Self> {
        let mut day = self.day as u64;
        let mut month = self.month as u8;
        let mut year = self.year;

        let nano = rhs.nanos;
        let seconds = rhs.seconds;

        let mut seconds = seconds;
        let mut nano = nano;

        while nano >= 1_000_000_000 {
            nano -= 1_000_000_000;
            seconds += 1;
        }

        while seconds >= 86400 {
            seconds -= 86400;
            day += 1;
        }

        while day > self.days_in_month() as u64 {
            day -= self.days_in_month() as u64;
            month += 1;
        }

        while month > 12 {
            month -= 12;
            year += 1;
        }

        Some(Self::new(day as u8, Month::from_u8(month).unwrap(), year))
    }

    pub fn sub(&self, rhs: Duration) -> Option<Self> {
        let mut day = self.day as i64;
        let mut month = self.month as i8;
        let mut year = self.year;

        let nano = rhs.nanos as i64;
        let seconds = rhs.seconds as i64;

        let mut seconds = seconds;
        let mut nano = nano;

        while nano < 0 {
            nano += 1_000_000_000;
            seconds -= 1;
        }

        while seconds < 0 {
            seconds += 86400;
            day -= 1;
        }

        while day < 1 {
            month -= 1;
            day += self.days_in_month() as i64;
        }

        while month < 1 {
            month += 12;
            year -= 1;
        }

        Some(Self::new(day as u8, Month::from_u8(month as u8).unwrap(), year))
    }

    pub fn as_calendar_date(&self) -> (i32, Month, u8) {
        (self.year, self.month, self.day)
    }

    pub fn as_ordinal_date(&self) -> (i32, u16) {
        (self.year, self.ordinal())
    }

    pub fn as_week_date(&self) -> (i32, u8, Weekday) {
        (self.year, self.week(), self.weekday())
    }
} impl From<Rtc> for Date {
    fn from(rtc: Rtc) -> Self {
        Self {
            day: rtc.day,
            month: Month::from_u8(rtc.month).unwrap(),
            year: rtc.year as i32,
        }
    }
} impl Display for Date {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}/{:02}/{:04}", self.day, self.month as u8, self.year)
    }
}

pub struct DateTime {
    time: Time,
    date: Date,
} #[allow(dead_code)] impl DateTime {
    pub fn new(
        nano: u32, seconds: u8, minutes: u8, hours: u8,
        day: u8, month: Month, year: i32,
    ) -> Self { Self {
        time: Time::new(nano, seconds, minutes, hours),
        date: Date::new(day, month, year),
    } }

    pub fn time(&self) -> Time { self.time }

    pub fn date(&self) -> Date { self.date }

    pub fn nano(&self) -> u32 { self.time.nano() }

    pub fn micro(&self) -> u32 { self.time.micro() }

    pub fn milli(&self) -> u32 { self.time.milli() }

    pub fn seconds(&self) -> u8 { self.time.seconds() }

    pub fn minutes(&self) -> u8 { self.time.minutes() }

    pub fn hours(&self) -> u8 { self.time.hours() }

    pub fn day(&self) -> u8 { self.date.day() }

    pub fn ordinal(&self) -> u16 { self.date.ordinal() }

    pub fn week(&self) -> u8 { self.date.week() }

    pub fn weekday(&self) -> Weekday { self.date.weekday() }

    pub fn month(&self) -> Month { self.date.month() }

    pub fn year(&self) -> i32 { self.date.year() }

    pub fn is_leap_year(&self) -> bool { self.date.is_leap_year() }

    pub fn days_in_month(&self) -> u8 { self.date.days_in_month() }

    pub fn as_hms(&self) -> (u8, u8, u8) { self.time.as_hms() }

    pub fn as_hms_milli(&self) -> (u8, u8, u8, u32) { self.time.as_hms_milli() }

    pub fn as_hms_micro(&self) -> (u8, u8, u8, u32) { self.time.as_hms_micro() }

    pub fn as_hms_nano(&self) -> (u8, u8, u8, u32) { self.time.as_hms_nano() }

    pub fn as_calendar_date(&self) -> (i32, Month, u8) { self.date.as_calendar_date() }

    pub fn as_ordinal_date(&self) -> (i32, u16) { self.date.as_ordinal_date() }

    pub fn as_week_date(&self) -> (i32, u8, Weekday) { self.date.as_week_date() }

    pub fn add(&self, rhs: Duration) -> Option<Self> {
        let new_time = self.time.add(rhs)?;
        let additional_days = new_time.hours / 24;
        let new_hours = new_time.hours % 24;
        let adjusted_time = Time::new(new_time.nano, new_time.seconds, new_time.minutes, new_hours);

        let mut day = self.date.day() as u64 + additional_days as u64;
        let mut month = self.date.month() as u8;
        let mut year = self.date.year();

        while day > Date::new(1, Month::from_u8(month).unwrap(), year).days_in_month() as u64 {
            day -= Date::new(1, Month::from_u8(month).unwrap(), year).days_in_month() as u64;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }

        let adjusted_date = Date::new(day as u8, Month::from_u8(month).unwrap(), year);

        Some(DateTime {
            time: adjusted_time,
            date: adjusted_date,
        })
    }
} impl From<Rtc> for DateTime {
    fn from(rtc: Rtc) -> Self {
        Self::new(
            0, rtc.seconds, rtc.minutes, rtc.hours,
            rtc.day, Month::from_u8(rtc.month).unwrap(), rtc.year as i32,
        )
    }
} impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}

pub trait TimeApi {
    /// Get the current date and time.
    fn now(&self) -> &DateTime;
}