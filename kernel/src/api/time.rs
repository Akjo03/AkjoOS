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

    pub fn from_hms(hours: u64, minutes: u64, seconds: u64) -> Self {
        Self {
            nanos: 0,
            seconds: hours * 3600 + minutes * 60 + seconds,
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
#[repr(u8)]
pub enum TimeOffset {
    /// -12:00
    Y = 0,
    /// -11:00
    X = 1,
    /// -10:00
    W = 2,
    /// -09:00
    V = 3,
    /// -09:30
    Vt = 4,
    /// -08:00
    U = 5,
    /// -07:00
    T = 6,
    /// -06:00
    S = 7,
    /// -05:00
    R = 8,
    /// -04:00
    Q = 9,
    /// -03:00
    P = 10,
    /// -03:30
    Pt = 11,
    /// -02:00
    O = 12,
    /// -01:00
    N = 13,
    /// -/+00:00
    Z = 14,
    /// +01:00
    A = 15,
    /// +02:00
    B = 16,
    /// +03:00
    C = 17,
    /// +03:30
    Ct = 18,
    /// +04:00
    D = 19,
    /// +04:30
    Dt = 20,
    /// +05:00
    E = 21,
    /// +05:30
    Et = 22,
    /// +05:45
    Ee = 23,
    /// +06:00
    F = 24,
    /// +06:30
    Ft = 25,
    /// +07:00
    G = 26,
    /// +08:00
    H = 27,
    /// +08:45
    Hh = 28,
    /// +09:00
    I = 29,
    /// +09:30
    It = 30,
    /// +10:00
    K = 31,
    /// +10:30
    Kt = 32,
    /// +11:00
    L = 33,
    /// +12:00
    M = 34,
    /// +12:45
    Mm = 35,
    /// +13:00
    Mt1 = 36,
    /// +14:00
    Mt2 = 37
} impl TimeOffset {
    pub fn from_u8(offset: u8) -> Option<Self> {
        match offset {
            0 => Some(TimeOffset::Y),
            1 => Some(TimeOffset::X),
            2 => Some(TimeOffset::W),
            3 => Some(TimeOffset::V),
            4 => Some(TimeOffset::Vt),
            5 => Some(TimeOffset::U),
            6 => Some(TimeOffset::T),
            7 => Some(TimeOffset::S),
            8 => Some(TimeOffset::R),
            9 => Some(TimeOffset::Q),
            10 => Some(TimeOffset::P),
            11 => Some(TimeOffset::Pt),
            12 => Some(TimeOffset::O),
            13 => Some(TimeOffset::N),
            14 => Some(TimeOffset::Z),
            15 => Some(TimeOffset::A),
            16 => Some(TimeOffset::B),
            17 => Some(TimeOffset::C),
            18 => Some(TimeOffset::Ct),
            19 => Some(TimeOffset::D),
            20 => Some(TimeOffset::Dt),
            21 => Some(TimeOffset::E),
            22 => Some(TimeOffset::Et),
            23 => Some(TimeOffset::Ee),
            24 => Some(TimeOffset::F),
            25 => Some(TimeOffset::Ft),
            26 => Some(TimeOffset::G),
            27 => Some(TimeOffset::H),
            28 => Some(TimeOffset::Hh),
            29 => Some(TimeOffset::I),
            30 => Some(TimeOffset::It),
            31 => Some(TimeOffset::K),
            32 => Some(TimeOffset::Kt),
            33 => Some(TimeOffset::L),
            34 => Some(TimeOffset::M),
            35 => Some(TimeOffset::Mm),
            36 => Some(TimeOffset::Mt1),
            37 => Some(TimeOffset::Mt2),
            _ => None
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn get_offset(&self) -> (bool, Duration) {
        match self {
            TimeOffset::Y => (false, Duration::from_hms(12, 0, 0)),
            TimeOffset::X => (false, Duration::from_hms(11, 0, 0)),
            TimeOffset::W => (false, Duration::from_hms(10, 0, 0)),
            TimeOffset::V => (false, Duration::from_hms(9, 0, 0)),
            TimeOffset::Vt => (false, Duration::from_hms(9, 30, 0)),
            TimeOffset::U => (false, Duration::from_hms(8, 0, 0)),
            TimeOffset::T => (false, Duration::from_hms(7, 0, 0)),
            TimeOffset::S => (false, Duration::from_hms(6, 0, 0)),
            TimeOffset::R => (false, Duration::from_hms(5, 0, 0)),
            TimeOffset::Q => (false, Duration::from_hms(4, 0, 0)),
            TimeOffset::P => (false, Duration::from_hms(3, 0, 0)),
            TimeOffset::Pt => (false, Duration::from_hms(3, 30, 0)),
            TimeOffset::O => (false, Duration::from_hms(2, 0, 0)),
            TimeOffset::N => (false, Duration::from_hms(1, 0, 0)),
            TimeOffset::Z => (true, Duration::from_hms(0, 0, 0)),
            TimeOffset::A => (true, Duration::from_hms(1, 0, 0)),
            TimeOffset::B => (true, Duration::from_hms(2, 0, 0)),
            TimeOffset::C => (true, Duration::from_hms(3, 0, 0)),
            TimeOffset::Ct => (true, Duration::from_hms(3, 30, 0)),
            TimeOffset::D => (true, Duration::from_hms(4, 0, 0)),
            TimeOffset::Dt => (true, Duration::from_hms(4, 30, 0)),
            TimeOffset::E => (true, Duration::from_hms(5, 0, 0)),
            TimeOffset::Et => (true, Duration::from_hms(5, 30, 0)),
            TimeOffset::Ee => (true, Duration::from_hms(5, 45, 0)),
            TimeOffset::F => (true, Duration::from_hms(6, 0, 0)),
            TimeOffset::Ft => (true, Duration::from_hms(6, 30, 0)),
            TimeOffset::G => (true, Duration::from_hms(7, 0, 0)),
            TimeOffset::H => (true, Duration::from_hms(8, 0, 0)),
            TimeOffset::Hh => (true, Duration::from_hms(8, 45, 0)),
            TimeOffset::I => (true, Duration::from_hms(9, 0, 0)),
            TimeOffset::It => (true, Duration::from_hms(9, 30, 0)),
            TimeOffset::K => (true, Duration::from_hms(10, 0, 0)),
            TimeOffset::Kt => (true, Duration::from_hms(10, 30, 0)),
            TimeOffset::L => (true, Duration::from_hms(11, 0, 0)),
            TimeOffset::M => (true, Duration::from_hms(12, 0, 0)),
            TimeOffset::Mm => (true, Duration::from_hms(12, 45, 0)),
            TimeOffset::Mt1 => (true, Duration::from_hms(13, 0, 0)),
            TimeOffset::Mt2 => (true, Duration::from_hms(14, 0, 0)),
        }
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

    pub fn from_rtc(rtc: Rtc) -> Self {
        Self {
            nano: 0,
            seconds: rtc.seconds,
            minutes: rtc.minutes,
            hours: rtc.hours,
        }
    }

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

    pub fn add(&self, rhs: Duration) -> Self {
        let total_nanos = self.nano as u64 + rhs.nanos;
        let extra_seconds = total_nanos / 1_000_000_000;
        let nano = (total_nanos % 1_000_000_000) as u32;

        let total_seconds = self.seconds as u64 + rhs.seconds + extra_seconds;
        let seconds = (total_seconds % 60) as u8;

        let total_minutes = self.minutes as u64 + (total_seconds / 60);
        let minutes = (total_minutes % 60) as u8;

        let hours = (self.hours as u64 + (total_minutes / 60)) % 24;

        Self { nano, seconds, minutes, hours: hours as u8 }
    }

    pub fn sub(&self, rhs: Duration) -> Self {
        let rhs_total_nanos = rhs.seconds * 1_000_000_000 + rhs.nanos;
        let self_total_nanos = self.seconds as u64 * 1_000_000_000 + self.nano as u64;
        let total_nanos = if self_total_nanos > rhs_total_nanos {
            self_total_nanos - rhs_total_nanos
        } else {
            0
        };

        let nano = (total_nanos % 1_000_000_000) as u32;
        let total_seconds = total_nanos / 1_000_000_000;

        let seconds = (total_seconds % 60) as u8;
        let total_minutes = self.minutes as u64 + (total_seconds / 60);

        let minutes = (total_minutes % 60) as u8;
        let hours = ((self.hours as u64 + (total_minutes / 60)) % 24) as u8;

        Self { nano, seconds, minutes, hours }
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

    pub fn from_rtc(rtc: Rtc) -> Self {
        Self {
            day: rtc.day,
            month: Month::from_u8(rtc.month).unwrap(),
            year: rtc.year as i32,
        }
    }

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

    pub fn add(&self, rhs: Duration) -> Self {
        let mut days_to_add = rhs.seconds / 86_400;
        let mut new_day = self.day;
        let mut new_month = self.month as u8;
        let mut new_year = self.year;

        while days_to_add > 0 {
            let days_in_current_month = Date::new(1, Month::from_u8(new_month).unwrap(), new_year).days_in_month();
            if new_day as u64 + days_to_add > days_in_current_month as u64 {
                days_to_add -= (days_in_current_month - new_day) as u64 + 1;
                new_day = 1;
                new_month += 1;
                if new_month > 12 {
                    new_month = 1;
                    new_year += 1;
                }
            } else {
                new_day += days_to_add as u8;
                days_to_add = 0;
            }
        }

        Date::new(new_day, Month::from_u8(new_month).unwrap(), new_year)
    }

    pub fn sub(&self, rhs: Duration) -> Self {
        let mut days_to_sub = rhs.seconds / 86_400;
        let mut new_day = self.day as i64;
        let mut new_month = self.month as u8;
        let mut new_year = self.year;

        while days_to_sub > 0 {
            if new_day as u64 <= days_to_sub {
                days_to_sub -= new_day as u64;
                new_month = if new_month == 1 { 12 } else { new_month - 1 };
                new_day = Date::new(1, Month::from_u8(new_month).unwrap(), new_year).days_in_month() as i64;
                if new_month == 12 {
                    new_year -= 1;
                }
            } else {
                new_day -= days_to_sub as i64;
                days_to_sub = 0;
            }
        }

        Date::new(new_day as u8, Month::from_u8(new_month).unwrap(), new_year)
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
} impl Display for Date {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}/{:02}/{:04}", self.day, self.month as u8, self.year)
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn from_rtc(rtc: Rtc) -> Self {
        Self::new(
            0, rtc.seconds, rtc.minutes, rtc.hours,
            rtc.day, Month::from_u8(rtc.month).unwrap(), rtc.year as i32,
        )
    }

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

    pub fn add(&self, rhs: Duration) -> Self {
        let new_time = self.time.add(rhs);
        let day_overflow = new_time.hours / 24;
        let new_date = self.date.add(Duration::from_days(day_overflow as u64));
        DateTime { time: Time::new(new_time.nano, new_time.seconds, new_time.minutes, new_time.hours % 24), date: new_date }
    }

    pub fn sub(&self, rhs: Duration) -> Self {
        let new_time = self.time.sub(rhs);
        let day_underflow = if new_time.hours > self.time.hours { 1 } else { 0 };
        let new_date = self.date.sub(Duration::from_days(day_underflow as u64));
        DateTime { time: Time::new(new_time.nano, new_time.seconds, new_time.minutes, new_time.hours % 24), date: new_date }
    }

    pub fn with_offset(&self, offset: TimeOffset) -> DateTime {
        let (positive, duration) = offset.get_offset();
        if positive {
            self.add(duration)
        } else {
            self.sub(duration)
        }
    }
} impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}

pub trait TimeApi {
    /// Get the current date and time.
    fn now(&self) -> DateTime;
    /// Get the current date and time with an offset.
    fn with_offset(&self, offset: TimeOffset) -> DateTime;
}