use chrono::{DateTime, Datelike, Local};
use colored::*;
use sunrise::sunrise_sunset;

#[derive(Debug)]
pub struct GoldenHourInfo {
    pub sunrise: DateTime<Local>,
    pub sunset: DateTime<Local>,
    pub golden_hour_morning_start: DateTime<Local>,
    pub golden_hour_morning_end: DateTime<Local>,
    pub golden_hour_evening_start: DateTime<Local>,
    pub golden_hour_evening_end: DateTime<Local>,
    pub blue_hour_morning_start: DateTime<Local>,
    pub blue_hour_morning_end: DateTime<Local>,
    pub blue_hour_evening_start: DateTime<Local>,
    pub blue_hour_evening_end: DateTime<Local>,
}

pub struct GoldenHourService {
    latitude: f64,
    longitude: f64,
}

impl GoldenHourService {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn calculate_golden_hours(&self, date: DateTime<Local>) -> GoldenHourInfo {
        let (sunrise_timestamp, sunset_timestamp) = sunrise_sunset(
            self.latitude,
            self.longitude,
            date.year(),
            date.month() as u32,
            date.day() as u32,
        );

        let sunrise = DateTime::from_timestamp(sunrise_timestamp, 0)
            .unwrap()
            .with_timezone(&Local);
        let sunset = DateTime::from_timestamp(sunset_timestamp, 0)
            .unwrap()
            .with_timezone(&Local);

        // Золотой час утром: за 1 час до восхода и 1 час после
        let golden_hour_morning_start = sunrise - chrono::Duration::hours(1);
        let golden_hour_morning_end = sunrise + chrono::Duration::hours(1);

        // Золотой час вечером: за 1 час до заката и 1 час после
        let golden_hour_evening_start = sunset - chrono::Duration::hours(1);
        let golden_hour_evening_end = sunset + chrono::Duration::hours(1);

        // Синий час утром: за 30 минут до восхода
        let blue_hour_morning_start = sunrise - chrono::Duration::minutes(30);
        let blue_hour_morning_end = sunrise;

        // Синий час вечером: за 30 минут после заката
        let blue_hour_evening_start = sunset;
        let blue_hour_evening_end = sunset + chrono::Duration::minutes(30);

        GoldenHourInfo {
            sunrise,
            sunset,
            golden_hour_morning_start,
            golden_hour_morning_end,
            golden_hour_evening_start,
            golden_hour_evening_end,
            blue_hour_morning_start,
            blue_hour_morning_end,
            blue_hour_evening_start,
            blue_hour_evening_end,
        }
    }

    pub fn get_current_lighting_condition(&self, current_time: DateTime<Local>) -> String {
        let golden_hours = self.calculate_golden_hours(current_time);

        if current_time >= golden_hours.golden_hour_morning_start
            && current_time <= golden_hours.golden_hour_morning_end
        {
            "Золотой час (утро)".to_string()
        } else if current_time >= golden_hours.golden_hour_evening_start
            && current_time <= golden_hours.golden_hour_evening_end
        {
            "Золотой час (вечер)".to_string()
        } else if current_time >= golden_hours.blue_hour_morning_start
            && current_time <= golden_hours.blue_hour_morning_end
        {
            "Синий час (утро)".to_string()
        } else if current_time >= golden_hours.blue_hour_evening_start
            && current_time <= golden_hours.blue_hour_evening_end
        {
            "Синий час (вечер)".to_string()
        } else if current_time >= golden_hours.sunrise && current_time <= golden_hours.sunset {
            "Дневное время".to_string()
        } else {
            "Ночное время".to_string()
        }
    }
}

pub fn print_golden_hour_info(info: &GoldenHourInfo) {
    println!("\n{}", "=== ЗОЛОТОЙ ЧАС И ВРЕМЯ СЪЕМКИ ===".bold().yellow());

    println!(
        "\n{}: {}",
        "Восход солнца".bold(),
        info.sunrise.format("%H:%M")
    );
    println!("{}: {}", "Закат солнца".bold(), info.sunset.format("%H:%M"));

    println!("\n{}:", "Золотой час (утро)".bold().yellow());
    println!(
        "  🌅 {} - {}",
        info.golden_hour_morning_start.format("%H:%M"),
        info.golden_hour_morning_end.format("%H:%M")
    );

    println!("\n{}:", "Золотой час (вечер)".bold().yellow());
    println!(
        "  🌇 {} - {}",
        info.golden_hour_evening_start.format("%H:%M"),
        info.golden_hour_evening_end.format("%H:%M")
    );

    println!("\n{}:", "Синий час (утро)".bold().blue());
    println!(
        "  🌊 {} - {}",
        info.blue_hour_morning_start.format("%H:%M"),
        info.blue_hour_morning_end.format("%H:%M")
    );

    println!("\n{}:", "Синий час (вечер)".bold().blue());
    println!(
        "  🌊 {} - {}",
        info.blue_hour_evening_start.format("%H:%M"),
        info.blue_hour_evening_end.format("%H:%M")
    );

    println!("\n{}:", "Рекомендации для фотографов".bold().green());
    println!("  📸 Золотой час - идеальное время для портретов и пейзажей");
    println!("  📸 Синий час - отлично подходит для городских пейзажей");
    println!("  📸 Планируйте съемку заранее, учитывая время восхода/заката");
}
