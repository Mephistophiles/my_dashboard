//! # Golden Hour Module
//!
//! Модуль для расчета золотого часа и синего часа для фотографии.
//! Предоставляет функциональность для определения оптимального времени съемки
//! с мягким естественным освещением.
//!
//! ## Основные компоненты
//!
//! - [`GoldenHourService`] - Сервис для расчета времени золотого часа
//! - [`GoldenHourInfo`] - Структура с информацией о времени восхода, заката и золотого часа
//!
//! ## Пример использования
//!
//! ```rust,no_run
//! use my_dashboard::golden_hour::GoldenHourService;
//! use chrono::Local;
//!
//! // Создаем сервис для Москвы
//! let service = GoldenHourService::new(55.7558, 37.6176);
//!
//! // Получаем информацию о золотом часе на сегодня
//! let current_time = Local::now();
//! let info = service.calculate_golden_hours(current_time);
//!
//! println!("Восход: {}", info.sunrise.format("%H:%M"));
//! println!("Закат: {}", info.sunset.format("%H:%M"));
//! println!("Золотой час утром: {}-{}",
//!     info.golden_hour_morning_start.format("%H:%M"),
//!     info.golden_hour_morning_end.format("%H:%M"));
//!
//! // Проверяем, сейчас ли золотой час
//! if service.is_golden_hour() {
//!     println!("Сейчас золотой час - идеальное время для съемки!");
//! }
//! ```

use chrono::{DateTime, Datelike, Local, NaiveDate};
use sunrise::{Coordinates, SolarDay, SolarEvent};
use crate::{get_current_time, is_demo_mode};

/// Информация о времени восхода, заката, золотого и синего часа
#[derive(Debug, Clone)]
pub struct GoldenHourInfo {
    /// Время восхода солнца
    pub sunrise: DateTime<Local>,
    /// Время заката солнца
    pub sunset: DateTime<Local>,
    /// Начало утреннего золотого часа
    pub golden_hour_morning_start: DateTime<Local>,
    /// Конец утреннего золотого часа
    pub golden_hour_morning_end: DateTime<Local>,
    /// Начало вечернего золотого часа
    pub golden_hour_evening_start: DateTime<Local>,
    /// Конец вечернего золотого часа
    pub golden_hour_evening_end: DateTime<Local>,
    /// Начало утреннего синего часа
    pub blue_hour_morning_start: DateTime<Local>,
    /// Конец утреннего синего часа
    pub blue_hour_morning_end: DateTime<Local>,
    /// Начало вечернего синего часа
    pub blue_hour_evening_start: DateTime<Local>,
    /// Конец вечернего синего часа
    pub blue_hour_evening_end: DateTime<Local>,
}

/// Сервис для расчета золотого часа и синего часа
///
/// Использует библиотеку `sunrise` для точных астрономических расчетов
/// времени восхода, заката и оптимальных условий освещения для фотографии.
pub struct GoldenHourService {
    latitude: f64,
    longitude: f64,
}

impl GoldenHourService {
    /// Создает новый экземпляр сервиса золотого часа
    ///
    /// # Аргументы
    ///
    /// * `latitude` - Широта в градусах (от -90 до 90)
    /// * `longitude` - Долгота в градусах (от -180 до 180)
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::golden_hour::GoldenHourService;
    ///
    /// let service = GoldenHourService::new(55.7558, 37.6176); // Москва
    /// ```
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// Рассчитывает время золотого и синего часа для указанной даты
    ///
    /// # Аргументы
    ///
    /// * `date` - Дата для расчета
    ///
    /// # Возвращает
    ///
    /// `GoldenHourInfo` - Полная информация о времени восхода, заката и золотого часа
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::golden_hour::GoldenHourService;
    /// use chrono::Local;
    ///
    /// let service = GoldenHourService::new(55.7558, 37.6176);
    /// let current_time = Local::now();
    /// let info = service.calculate_golden_hours(current_time);
    ///
    /// println!("Восход: {}", info.sunrise.format("%H:%M"));
    /// println!("Закат: {}", info.sunset.format("%H:%M"));
    /// ```
    pub fn calculate_golden_hours(&self, date: DateTime<Local>) -> GoldenHourInfo {
        // В DEMO режиме используем фиксированную дату для стабильности тестов
        let demo_mode = is_demo_mode();

        let calculation_date = if demo_mode {
            // Используем фиксированную дату для стабильности (ночное время)
            get_current_time()
        } else {
            date
        };

        // Создаем координаты
        let coords = Coordinates::new(self.latitude, self.longitude).expect("Invalid coordinates");

        // Создаем дату
        let naive_date =
            NaiveDate::from_ymd_opt(calculation_date.year(), calculation_date.month(), calculation_date.day()).expect("Invalid date");

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

    /// Проверяет, является ли текущее время золотым часом
    ///
    /// # Возвращает
    ///
    /// `bool` - `true` если сейчас золотой час, `false` в противном случае
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::golden_hour::GoldenHourService;
    ///
    /// let service = GoldenHourService::new(55.7558, 37.6176);
    /// if service.is_golden_hour() {
    ///     println!("Сейчас золотой час - идеальное время для съемки!");
    /// }
    /// ```
    pub fn is_golden_hour(&self) -> bool {
        let current_time = chrono::Local::now();
        let golden_hours = self.calculate_golden_hours(current_time);

        (current_time >= golden_hours.golden_hour_morning_start
            && current_time <= golden_hours.golden_hour_morning_end)
            || (current_time >= golden_hours.golden_hour_evening_start
                && current_time <= golden_hours.golden_hour_evening_end)
    }

    /// Определяет текущие условия освещения для указанного времени
    ///
    /// # Аргументы
    ///
    /// * `current_time` - Время для проверки условий освещения
    ///
    /// # Возвращает
    ///
    /// `String` - Описание текущих условий освещения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::golden_hour::GoldenHourService;
    /// use chrono::Local;
    ///
    /// let service = GoldenHourService::new(55.7558, 37.6176);
    /// let current_time = Local::now();
    /// let condition = service.get_current_lighting_condition(current_time);
    /// println!("Текущие условия: {}", condition);
    /// ```
    pub fn get_current_lighting_condition(&self, current_time: DateTime<Local>) -> String {
        // В DEMO режиме используем фиксированное время для стабильности тестов
        let demo_mode = is_demo_mode();

        let calculation_time = if demo_mode {
            // Используем фиксированное время для стабильности (ночное время)
            get_current_time()
        } else {
            current_time
        };

        let golden_hours = self.calculate_golden_hours(calculation_time);

        // Сначала проверяем синий час
        if calculation_time >= golden_hours.blue_hour_morning_start
            && calculation_time <= golden_hours.blue_hour_morning_end
        {
            "Синий час (утро)".to_string()
        } else if calculation_time >= golden_hours.blue_hour_evening_start
            && calculation_time <= golden_hours.blue_hour_evening_end
        {
            "Синий час (вечер)".to_string()
        } else if calculation_time >= golden_hours.golden_hour_morning_start
            && calculation_time <= golden_hours.golden_hour_morning_end
        {
            "Золотой час (утро)".to_string()
        } else if calculation_time >= golden_hours.golden_hour_evening_start
            && calculation_time <= golden_hours.golden_hour_evening_end
        {
            "Золотой час (вечер)".to_string()
        } else if calculation_time >= golden_hours.sunrise && calculation_time <= golden_hours.sunset {
            "Дневное время".to_string()
        } else {
            "Ночное время".to_string()
        }
    }
}

/// Выводит информацию о золотом часе в консоль
///
/// Функция для удобного вывода всей информации о времени восхода, заката,
/// золотого и синего часа в отформатированном виде.
///
/// # Аргументы
///
/// * `service` - Сервис золотого часа
///
/// # Пример
///
/// ```rust,no_run
/// use my_dashboard::golden_hour::{GoldenHourService, print_golden_hour_info};
///
/// let service = GoldenHourService::new(55.7558, 37.6176);
/// print_golden_hour_info(&service);
/// ```
pub fn print_golden_hour_info(service: &GoldenHourService) {
    // В DEMO режиме используем фиксированное время для стабильности тестов
    let demo_mode = is_demo_mode();

    let current_time = if demo_mode {
        // Используем фиксированное время для стабильности (ночное время)
        get_current_time()
    } else {
        chrono::Local::now()
    };

    let info = service.calculate_golden_hours(current_time);
    let current_condition = service.get_current_lighting_condition(current_time);

    println!(
        "🌅 Восход: {} | 🌆 Закат: {} | 🌅 Золотой час утро: {}-{} | 🌆 Золотой час вечер: {}-{} | 🌅 Синий час утро: {}-{} | 🌆 Синий час вечер: {}-{}",
        info.sunrise.format("%H:%M"),
        info.sunset.format("%H:%M"),
        info.golden_hour_morning_start.format("%H:%M"),
        info.golden_hour_morning_end.format("%H:%M"),
        info.golden_hour_evening_start.format("%H:%M"),
        info.golden_hour_evening_end.format("%H:%M"),
        info.blue_hour_morning_start.format("%H:%M"),
        info.blue_hour_morning_end.format("%H:%M"),
        info.blue_hour_evening_start.format("%H:%M"),
        info.blue_hour_evening_end.format("%H:%M")
    );

    println!("💡 Текущие условия освещения: {}", current_condition);
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
        let naive_datetime = NaiveDateTime::new(
            naive_date,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );
        Local.from_local_datetime(&naive_datetime).unwrap()
    }

    fn create_winter_date() -> DateTime<Local> {
        // Зимний день для тестирования коротких дней
        let naive_date = NaiveDate::from_ymd_opt(2024, 12, 21).unwrap();
        let naive_datetime = NaiveDateTime::new(
            naive_date,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );
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
        assert_eq!(
            info.golden_hour_morning_end - info.golden_hour_morning_start,
            chrono::Duration::hours(2)
        );
        assert_eq!(
            info.golden_hour_evening_end - info.golden_hour_evening_start,
            chrono::Duration::hours(2)
        );

        // Проверяем синие часы
        assert_eq!(
            info.blue_hour_morning_end - info.blue_hour_morning_start,
            chrono::Duration::minutes(30)
        );
        assert_eq!(
            info.blue_hour_evening_end - info.blue_hour_evening_start,
            chrono::Duration::minutes(30)
        );
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

        println!(
            "\nblue_hour_morning_start: {}",
            info.blue_hour_morning_start
        );
        println!("blue_hour_morning_end:   {}", info.blue_hour_morning_end);
        println!(
            "golden_hour_morning_start: {}",
            info.golden_hour_morning_start
        );
        println!(
            "golden_hour_morning_end:   {}",
            info.golden_hour_morning_end
        );
        println!("sunrise: {}", info.sunrise);
        println!("sunset:  {}", info.sunset);

        // Для золотого часа утром используем время сразу после окончания синего часа
        let morning_golden = service.get_current_lighting_condition(
            info.blue_hour_morning_end + chrono::Duration::minutes(1),
        );
        assert_eq!(morning_golden, "Золотой час (утро)");

        let evening_golden = service.get_current_lighting_condition(
            info.golden_hour_evening_start + chrono::Duration::minutes(30),
        );
        assert_eq!(evening_golden, "Золотой час (вечер)");

        // Проверяем синие часы - используем blue_hour_morning_start + 5 минут
        let morning_blue = service.get_current_lighting_condition(
            info.blue_hour_morning_start + chrono::Duration::minutes(5),
        );
        println!(
            "test time for morning blue: {}",
            info.blue_hour_morning_start + chrono::Duration::minutes(5)
        );
        println!("lighting condition: {}", morning_blue);
        assert_eq!(morning_blue, "Синий час (утро)");

        let evening_blue = service.get_current_lighting_condition(
            info.blue_hour_evening_start + chrono::Duration::minutes(5),
        );
        assert_eq!(evening_blue, "Синий час (вечер)");

        // Проверяем дневное и ночное время
        let daytime =
            service.get_current_lighting_condition(info.sunrise + chrono::Duration::hours(6));
        assert_eq!(daytime, "Дневное время");

        // Для ночного времени используем время до начала синего часа утром
        let nighttime = service.get_current_lighting_condition(
            info.blue_hour_morning_start - chrono::Duration::hours(1),
        );
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
        assert!((0..=23).contains(&info.sunrise.hour()));
        assert!((0..=23).contains(&info.sunset.hour()));
        assert!((0..=23).contains(&info.golden_hour_morning_start.hour()));
        assert!((0..=23).contains(&info.golden_hour_evening_start.hour()));

        // Проверяем, что золотые часы не пересекаются
        assert!(info.golden_hour_morning_end < info.golden_hour_evening_start);
    }
}
