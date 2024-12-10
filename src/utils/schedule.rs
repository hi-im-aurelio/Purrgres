use std::time::Duration;

pub enum Schedule {
    OneMonth,
    SevenDays,
    SixDays,
    OneDay,
    TwelveHours,
    OneHour,
    ThirtyMinutes,
    OneMinute,
}

impl Schedule {
    pub fn to_duration(&self) -> Duration {
        match self {
            Schedule::OneMonth => Duration::from_secs(30 * 24 * 60 * 60), // 1 mes
            Schedule::SevenDays => Duration::from_secs(7 * 24 * 60 * 60), // 7 dias
            Schedule::SixDays => Duration::from_secs(6 * 24 * 60 * 60),   // 6 dias
            Schedule::OneDay => Duration::from_secs(24 * 60 * 60),        // 1 dia
            Schedule::TwelveHours => Duration::from_secs(12 * 60 * 60),   // 12 horas
            Schedule::OneHour => Duration::from_secs(60 * 60),            // 1 hora
            Schedule::ThirtyMinutes => Duration::from_secs(30 * 60),      // 30 min
            Schedule::OneMinute => Duration::from_secs(60),               // 1 min
        }
    }
}
