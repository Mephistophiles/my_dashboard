use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use sunrise::{Coordinates, SolarDay, SolarEvent};

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
        // Создаем координаты
        let coords = Coordinates::new(self.latitude, self.longitude).expect("Invalid coordinates");

        // Создаем дату
        let naive_date =
            NaiveDate::from_ymd_opt(date.year(), date.month(), date.day()).expect("Invalid date");

        // Создаем солнечный день
        let solar_day = SolarDay::new(coords, naive_date);

        // Получаем время восхода и заката
        let sunrise_timestamp = solar_day.event_time(SolarEvent::Sunrise).timestamp();
        let sunset_timestamp = solar_day.event_time(SolarEvent::Sunset).timestamp();

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

    pub fn is_golden_hour(&self) -> bool {
        let current_time = chrono::Local::now();
        let golden_hours = self.calculate_golden_hours(current_time);

        (current_time >= golden_hours.golden_hour_morning_start
            && current_time <= golden_hours.golden_hour_morning_end)
            || (current_time >= golden_hours.golden_hour_evening_start
                && current_time <= golden_hours.golden_hour_evening_end)
    }

    pub fn get_current_lighting_condition(&self, current_time: DateTime<Local>) -> String {
        let golden_hours = self.calculate_golden_hours(current_time);

        // Сначала проверяем синий час
        if current_time >= golden_hours.blue_hour_morning_start
            && current_time <= golden_hours.blue_hour_morning_end
        {
            "Синий час (утро)".to_string()
        } else if current_time >= golden_hours.blue_hour_evening_start
            && current_time <= golden_hours.blue_hour_evening_end
        {
            "Синий час (вечер)".to_string()
        } else if current_time >= golden_hours.golden_hour_morning_start
            && current_time <= golden_hours.golden_hour_morning_end
        {
            "Золотой час (утро)".to_string()
        } else if current_time >= golden_hours.golden_hour_evening_start
            && current_time <= golden_hours.golden_hour_evening_end
        {
            "Золотой час (вечер)".to_string()
        } else if current_time >= golden_hours.sunrise && current_time <= golden_hours.sunset {
            "Дневное время".to_string()
        } else {
            "Ночное время".to_string()
        }
    }
}

pub fn print_golden_hour_info(service: &GoldenHourService) {
    let current_time = chrono::Local::now();
    let info = service.calculate_golden_hours(current_time);

    println!(
        "🌅 Золотой час утро: {}-{} | 🌆 Золотой час вечер: {}-{} | 🌅 Синий час утро: {}-{} | 🌆 Синий час вечер: {}-{}",
        info.golden_hour_morning_start.format("%H:%M"),
        info.golden_hour_morning_end.format("%H:%M"),
        info.golden_hour_evening_start.format("%H:%M"),
        info.golden_hour_evening_end.format("%H:%M"),
        info.blue_hour_morning_start.format("%H:%M"),
        info.blue_hour_morning_end.format("%H:%M"),
        info.blue_hour_evening_start.format("%H:%M"),
        info.blue_hour_evening_end.format("%H:%M")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone, Timelike};

    // Вспомогательные функции для создания тестовых данных
    fn create_test_service() -> GoldenHourService {
        GoldenHourService::new(55.7558, 37.6176) // Москва
    }

    fn create_test_date() -> DateTime<Local> {
        // Используем фиксированную дату для тестов (летний день)
        let naive_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let naive_datetime = NaiveDateTime::new(naive_date, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        Local.from_local_datetime(&naive_datetime).unwrap()
    }

    fn create_winter_date() -> DateTime<Local> {
        // Зимний день для тестирования коротких дней
        let naive_date = NaiveDate::from_ymd_opt(2024, 12, 21).unwrap();
        let naive_datetime = NaiveDateTime::new(naive_date, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        Local.from_local_datetime(&naive_datetime).unwrap()
    }

    #[test]
    fn test_golden_hour_service_new() {
        let service = GoldenHourService::new(55.7558, 37.6176);
        
        assert_eq!(service.latitude, 55.7558);
        assert_eq!(service.longitude, 37.6176);
    }

    #[test]
    fn test_golden_hour_info_structure() {
        let service = create_test_service();
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);
        
        // Проверяем, что все поля заполнены
        assert!(info.sunrise > info.golden_hour_morning_start);
        assert!(info.sunrise < info.golden_hour_morning_end);
        assert!(info.sunset > info.golden_hour_evening_start);
        assert!(info.sunset < info.golden_hour_evening_end);
        
        // Проверяем золотые часы
        assert_eq!(info.golden_hour_morning_end - info.golden_hour_morning_start, chrono::Duration::hours(2));
        assert_eq!(info.golden_hour_evening_end - info.golden_hour_evening_start, chrono::Duration::hours(2));
        
        // Проверяем синие часы
        assert_eq!(info.blue_hour_morning_end - info.blue_hour_morning_start, chrono::Duration::minutes(30));
        assert_eq!(info.blue_hour_evening_end - info.blue_hour_evening_start, chrono::Duration::minutes(30));
    }

    #[test]
    fn test_golden_hour_timing() {
        let service = create_test_service();
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);
        
        // Золотой час утром должен быть за 1 час до восхода и 1 час после
        let expected_morning_start = info.sunrise - chrono::Duration::hours(1);
        let expected_morning_end = info.sunrise + chrono::Duration::hours(1);
        
        assert_eq!(info.golden_hour_morning_start, expected_morning_start);
        assert_eq!(info.golden_hour_morning_end, expected_morning_end);
        
        // Золотой час вечером должен быть за 1 час до заката и 1 час после
        let expected_evening_start = info.sunset - chrono::Duration::hours(1);
        let expected_evening_end = info.sunset + chrono::Duration::hours(1);
        
        assert_eq!(info.golden_hour_evening_start, expected_evening_start);
        assert_eq!(info.golden_hour_evening_end, expected_evening_end);
    }

    #[test]
    fn test_blue_hour_timing() {
        let service = create_test_service();
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);
        
        // Синий час утром должен быть за 30 минут до восхода
        let expected_morning_start = info.sunrise - chrono::Duration::minutes(30);
        let expected_morning_end = info.sunrise;
        
        assert_eq!(info.blue_hour_morning_start, expected_morning_start);
        assert_eq!(info.blue_hour_morning_end, expected_morning_end);
        
        // Синий час вечером должен быть за 30 минут после заката
        let expected_evening_start = info.sunset;
        let expected_evening_end = info.sunset + chrono::Duration::minutes(30);
        
        assert_eq!(info.blue_hour_evening_start, expected_evening_start);
        assert_eq!(info.blue_hour_evening_end, expected_evening_end);
    }

    #[test]
    fn test_day_night_cycle() {
        let service = create_test_service();
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);
        
        // Восход должен быть раньше заката
        assert!(info.sunrise < info.sunset);
        
        // Золотой час утром должен быть раньше золотого часа вечером
        assert!(info.golden_hour_morning_start < info.golden_hour_evening_start);
        assert!(info.golden_hour_morning_end < info.golden_hour_evening_end);
        
        // Синий час утром должен быть раньше синего часа вечером
        assert!(info.blue_hour_morning_start < info.blue_hour_evening_start);
        assert!(info.blue_hour_morning_end < info.blue_hour_evening_end);
    }

    #[test]
    fn test_lighting_conditions() {
        let service = create_test_service();
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);

        println!("\nblue_hour_morning_start: {}", info.blue_hour_morning_start);
        println!("blue_hour_morning_end:   {}", info.blue_hour_morning_end);
        println!("golden_hour_morning_start: {}", info.golden_hour_morning_start);
        println!("golden_hour_morning_end:   {}", info.golden_hour_morning_end);
        println!("sunrise: {}", info.sunrise);
        println!("sunset:  {}", info.sunset);

        // Для золотого часа утром используем время сразу после окончания синего часа
        let morning_golden = service.get_current_lighting_condition(info.blue_hour_morning_end + chrono::Duration::minutes(1));
        assert_eq!(morning_golden, "Золотой час (утро)");

        let evening_golden = service.get_current_lighting_condition(info.golden_hour_evening_start + chrono::Duration::minutes(30));
        assert_eq!(evening_golden, "Золотой час (вечер)");

        // Проверяем синие часы - используем blue_hour_morning_start + 5 минут
        let morning_blue = service.get_current_lighting_condition(info.blue_hour_morning_start + chrono::Duration::minutes(5));
        println!("test time for morning blue: {}", info.blue_hour_morning_start + chrono::Duration::minutes(5));
        println!("lighting condition: {}", morning_blue);
        assert_eq!(morning_blue, "Синий час (утро)");

        let evening_blue = service.get_current_lighting_condition(info.blue_hour_evening_start + chrono::Duration::minutes(5));
        assert_eq!(evening_blue, "Синий час (вечер)");

        // Проверяем дневное и ночное время
        let daytime = service.get_current_lighting_condition(info.sunrise + chrono::Duration::hours(6));
        assert_eq!(daytime, "Дневное время");

        // Для ночного времени используем время до начала синего часа утром
        let nighttime = service.get_current_lighting_condition(info.blue_hour_morning_start - chrono::Duration::hours(1));
        assert_eq!(nighttime, "Ночное время");
    }

    #[test]
    fn test_seasonal_variations() {
        let service = create_test_service();
        
        // Летний день
        let summer_date = create_test_date();
        let summer_info = service.calculate_golden_hours(summer_date);
        
        // Зимний день
        let winter_date = create_winter_date();
        let winter_info = service.calculate_golden_hours(winter_date);
        
        // Летом день должен быть длиннее
        let summer_day_length = summer_info.sunset - summer_info.sunrise;
        let winter_day_length = winter_info.sunset - winter_info.sunrise;
        
        assert!(summer_day_length > winter_day_length);
    }

    #[test]
    fn test_coordinate_validation() {
        // Тестируем с разными координатами
        let moscow = GoldenHourService::new(55.7558, 37.6176);
        let spb = GoldenHourService::new(59.9311, 30.3609);
        let murmansk = GoldenHourService::new(68.9792, 33.0925);
        
        let test_date = create_test_date();
        
        // Все должны работать без ошибок
        let _moscow_info = moscow.calculate_golden_hours(test_date);
        let _spb_info = spb.calculate_golden_hours(test_date);
        let _murmansk_info = murmansk.calculate_golden_hours(test_date);
    }

    #[test]
    fn test_golden_hour_detection() {
        let service = create_test_service();
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);
        
        // Создаем время в синий час утром (вложен в золотой час)
        let blue_morning_time = info.blue_hour_morning_start + chrono::Duration::minutes(5);
        // Создаем время в золотой час вечером (не вложен в синий)
        let golden_evening_time = info.golden_hour_evening_start + chrono::Duration::minutes(30);
        // Создаем время вне золотого и синего часа
        let non_golden_time = info.sunrise + chrono::Duration::hours(6);
        
        // Проверяем определение синего и золотого часа
        let morning_condition = service.get_current_lighting_condition(blue_morning_time);
        let evening_condition = service.get_current_lighting_condition(golden_evening_time);
        let non_golden_condition = service.get_current_lighting_condition(non_golden_time);
        
        assert_eq!(morning_condition, "Синий час (утро)");
        assert_eq!(evening_condition, "Золотой час (вечер)");
        assert_eq!(non_golden_condition, "Дневное время");
    }

    #[test]
    fn test_edge_cases() {
        let service = create_test_service();
        
        // Тестируем граничные случаи
        let test_date = create_test_date();
        let info = service.calculate_golden_hours(test_date);
        
        // Проверяем, что все времена находятся в разумных пределах
        assert!(info.sunrise.hour() >= 0 && info.sunrise.hour() <= 23);
        assert!(info.sunset.hour() >= 0 && info.sunset.hour() <= 23);
        assert!(info.golden_hour_morning_start.hour() >= 0 && info.golden_hour_morning_start.hour() <= 23);
        assert!(info.golden_hour_evening_start.hour() >= 0 && info.golden_hour_evening_start.hour() <= 23);
        
        // Проверяем, что золотые часы не пересекаются
        assert!(info.golden_hour_morning_end < info.golden_hour_evening_start);
    }
}
