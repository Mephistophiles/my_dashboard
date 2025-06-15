//! # Weather Module
//!
//! Модуль для анализа погодных условий для фотографии.
//! Предоставляет функциональность для получения прогноза погоды,
//! анализа условий съемки и оценки пригодности для астрофотографии.
//!
//! ## Основные компоненты
//!
//! - [`WeatherService`] - Сервис для получения данных о погоде
//! - [`WeatherForecast`] - Структура прогноза погоды
//! - [`WeatherAnalysis`] - Результат анализа погодных условий
//! - [`AstrophotographyAnalysis`] - Анализ условий для астрофотографии
//!
//! ## Пример использования
//!
//! ```rust
//! use my_dashboard::weather::{WeatherService, analyze_weather_for_photography};
//!
//! // Создаем сервис погоды
//! let weather_service = WeatherService::new(
//!     "your_api_key".to_string(),
//!     "Moscow".to_string(),
//! );
//!
//! // Для асинхронного использования:
//! // #[tokio::main]
//! // async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! //     let forecast = weather_service.get_weather_forecast().await?;
//! //     let analysis = analyze_weather_for_photography(&forecast);
//! //     println!("Оценка условий: {}/10", analysis.overall_score);
//! //     Ok(())
//! // }
//! ```

use anyhow::Result;
use chrono::{DateTime, Timelike, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub cloud_cover: f64,
    pub visibility: f64,
    pub precipitation_probability: f64,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub hourly: Vec<WeatherData>,
}

// Структуры для парсинга ответа OpenWeatherMap API
#[derive(Debug, Deserialize)]
struct CurrentWeatherResponse {
    main: CurrentWeatherMain,
    wind: CurrentWeatherWind,
    clouds: CurrentWeatherClouds,
    visibility: f64,
    weather: Vec<OpenWeatherCondition>,
}

#[derive(Debug, Deserialize)]
struct CurrentWeatherMain {
    temp: f64,
    humidity: f64,
}

#[derive(Debug, Deserialize)]
struct CurrentWeatherWind {
    speed: f64,
}

#[derive(Debug, Deserialize)]
struct CurrentWeatherClouds {
    all: f64,
}

#[derive(Debug, Deserialize)]
struct OpenWeatherCondition {
    description: String,
}

pub struct WeatherService {
    api_key: String,
    city: String,
    demo_mode: bool,
}

impl WeatherService {
    pub fn new(api_key: String, city: String) -> Self {
        debug!("Создание WeatherService для города: {}", city);

        // Проверяем DEMO режим
        let demo_mode = std::env::var("DEMO_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        if demo_mode {
            warn!("Включен DEMO режим - используются демонстрационные данные");
        }

        Self {
            api_key,
            city,
            demo_mode,
        }
    }

    pub async fn get_weather_forecast(&self) -> Result<WeatherForecast> {
        debug!("Запрос прогноза погоды для города: {}", self.city);

        // Если включен DEMO режим или используется demo_key, возвращаем моковые данные
        if self.demo_mode || self.api_key == "demo_key" {
            warn!("Используются демонстрационные данные погоды");
            return self.get_mock_forecast();
        }

        // Получаем координаты города
        let coords = self.get_city_coordinates().await?;
        debug!(
            "Координаты города {}: lat={}, lon={}",
            self.city, coords.lat, coords.lon
        );

        // Используем бесплатный Current Weather API вместо OneCall
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={}",
            coords.lat, coords.lon, self.api_key
        );

        debug!("Запрос к OpenWeather API: {}", url);
        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_message = match status.as_u16() {
                401 => {
                    "Неверный API ключ. Получите бесплатный ключ на https://openweathermap.org/api"
                        .to_string()
                }
                429 => "Превышен лимит запросов. Попробуйте позже.".to_string(),
                404 => {
                    format!("Город '{}' не найден", self.city)
                }
                _ => {
                    format!("HTTP ошибка {} при получении данных погоды", status)
                }
            };
            warn!("Ошибка API: {}", error_message);
            return Err(anyhow::anyhow!(error_message));
        }

        let weather_response: CurrentWeatherResponse = response.json().await?;
        info!(
            "Получены данные погоды: {}°C, облачность {}%",
            weather_response.main.temp, weather_response.clouds.all
        );

        // Создаем прогноз на основе текущих данных БЕЗ случайных вариаций
        let mut forecast = WeatherForecast { hourly: Vec::new() };

        // Генерируем прогноз на 24 часа с реалистичными суточными циклами
        let current_time = chrono::Utc::now();
        let base_temp = weather_response.main.temp;

        for hour in 0..24 {
            // Создаем реалистичные суточные вариации температуры БЕЗ случайности
            let hour_of_day = (current_time.hour() + hour as u32) % 24;
            let temp_variation = match hour_of_day {
                6..=8 => -2.0,   // Утро прохладнее
                9..=11 => -1.0,  // Начало дня
                12..=16 => 0.0,  // День - базовая температура
                17..=19 => -1.0, // Вечер
                20..=22 => -2.0, // Поздний вечер
                _ => -3.0,       // Ночь холоднее
            };

            let temperature = base_temp + temp_variation;

            // Суточные вариации других параметров БЕЗ случайности
            let humidity_variation = match hour_of_day {
                6..=8 => -5.0,   // Утро - меньше влажности
                12..=16 => 5.0,  // День - больше влажности
                20..=22 => -3.0, // Вечер
                _ => 0.0,
            };

            let wind_variation = match hour_of_day {
                12..=16 => 1.0, // День - ветер сильнее
                _ => 0.0,
            };

            let cloud_variation = match hour_of_day {
                6..=8 => -10.0, // Утро - меньше облаков
                12..=16 => 5.0, // День - больше облаков
                _ => 0.0,
            };

            let weather_data = WeatherData {
                temperature: temperature.clamp(-20.0, 50.0), // Ограничиваем разумными пределами
                humidity: (weather_response.main.humidity + humidity_variation).clamp(0.0, 100.0),
                wind_speed: (weather_response.wind.speed + wind_variation).max(0.0),
                cloud_cover: (weather_response.clouds.all + cloud_variation).clamp(0.0, 100.0),
                visibility: weather_response.visibility / 1000.0, // конвертируем в км
                precipitation_probability: if weather_response.clouds.all > 70.0 {
                    20.0
                } else {
                    5.0
                },
                description: weather_response
                    .weather
                    .first()
                    .map(|w| w.description.clone())
                    .unwrap_or_else(|| "Неизвестно".to_string()),
                timestamp: current_time + chrono::Duration::hours(hour),
            };
            forecast.hourly.push(weather_data);
        }

        debug!("Сгенерирован прогноз на 24 часа с суточными циклами");
        Ok(forecast)
    }

    async fn get_city_coordinates(&self) -> Result<CityCoordinates> {
        let url = format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            self.city, self.api_key
        );

        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow::anyhow!(
                "HTTP ошибка {} при получении координат города '{}'",
                status,
                self.city
            ));
        }

        let coords: Vec<CityCoordinates> = response.json().await?;

        if let Some(coord) = coords.first() {
            Ok(coord.clone())
        } else {
            Err(anyhow::anyhow!("Город '{}' не найден", self.city))
        }
    }

    fn get_mock_forecast(&self) -> Result<WeatherForecast> {
        // Моковые данные для демонстрации (только в DEMO режиме)
        let mut forecast = WeatherForecast { hourly: Vec::new() };

        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 15.0 + (hour as f64 * 0.5) - 6.0, // Температура от 9 до 21 градуса
                humidity: 60.0 + (hour as f64 * 2.0) % 40.0,
                wind_speed: 5.0 + (hour as f64 * 0.3) % 15.0,
                cloud_cover: if !(6..=18).contains(&hour) {
                    20.0
                } else {
                    60.0
                },
                visibility: 10.0 - (hour as f64 * 0.1) % 5.0,
                precipitation_probability: if hour > 12 && hour < 18 { 30.0 } else { 5.0 },
                description: match hour {
                    6..=8 => "Ясное утро".to_string(),
                    9..=11 => "Солнечно".to_string(),
                    12..=14 => "Переменная облачность".to_string(),
                    15..=17 => "Облачно".to_string(),
                    18..=20 => "Закат".to_string(),
                    _ => "Ночь".to_string(),
                },
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            forecast.hourly.push(weather_data);
        }

        Ok(forecast)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct CityCoordinates {
    lat: f64,
    lon: f64,
}

pub fn analyze_weather_for_photography(forecast: &WeatherForecast) -> WeatherAnalysis {
    let mut analysis = WeatherAnalysis {
        overall_score: 0.0,
        recommendations: Vec::new(),
        best_hours: Vec::new(),
        concerns: Vec::new(),
    };

    // Анализируем каждый час
    for (hour, weather) in forecast.hourly.iter().enumerate() {
        let mut hour_score = 0.0;
        let mut hour_recommendations = Vec::new();
        let mut hour_concerns = Vec::new();

        // Оценка температуры
        if weather.temperature >= 10.0 && weather.temperature <= 25.0 {
            hour_score += 2.0;
        } else {
            hour_concerns.push(format!("Неудобная температура: {}°C", weather.temperature));
        }

        // Оценка ветра
        if weather.wind_speed < 10.0 {
            hour_score += 2.0;
        } else {
            hour_concerns.push(format!("Сильный ветер: {} м/с", weather.wind_speed));
        }

        // Оценка облачности
        if weather.cloud_cover < 30.0 {
            hour_score += 3.0;
            hour_recommendations.push("Отличная видимость".to_string());
        } else if weather.cloud_cover < 70.0 {
            hour_score += 1.5;
            hour_recommendations.push("Хорошие условия для съемки".to_string());
        } else {
            hour_concerns.push(format!("Высокая облачность: {}%", weather.cloud_cover));
        }

        // Оценка видимости
        if weather.visibility > 8.0 {
            hour_score += 2.0;
        } else {
            hour_concerns.push(format!("Плохая видимость: {} км", weather.visibility));
        }

        // Оценка осадков
        if weather.precipitation_probability < 20.0 {
            hour_score += 1.0;
        } else {
            hour_concerns.push(format!(
                "Вероятность осадков: {}%",
                weather.precipitation_probability
            ));
        }

        // Специальные условия для фотографии
        if (6..=8).contains(&hour) {
            hour_score += 2.0; // Золотой час утром
            hour_recommendations.push("Золотой час - идеальное время для съемки".to_string());
        } else if (18..=20).contains(&hour) {
            hour_score += 2.0; // Золотой час вечером
            hour_recommendations.push("Золотой час - идеальное время для съемки".to_string());
        }

        if hour_score >= 7.0 {
            analysis.best_hours.push(hour);
        }

        analysis.overall_score += hour_score;

        // Добавляем concerns в общий список, если они есть
        analysis.concerns.extend(hour_concerns);
    }

    analysis.overall_score /= 24.0;

    // Общие рекомендации
    if analysis.overall_score >= 7.0 {
        analysis
            .recommendations
            .push("Отличные условия для фотографии!".to_string());
    } else if analysis.overall_score >= 5.0 {
        analysis
            .recommendations
            .push("Хорошие условия для съемки".to_string());
    } else {
        analysis
            .recommendations
            .push("Условия не идеальны для фотографии".to_string());
    }

    analysis
}

pub fn analyze_astrophotography_conditions(forecast: &WeatherForecast) -> AstrophotographyAnalysis {
    let mut analysis = AstrophotographyAnalysis {
        is_suitable: true,
        cloud_cover_issues: Vec::new(),
        recommendations: Vec::new(),
        best_hours: Vec::new(),
        concerns: Vec::new(),
    };

    // Анализируем условия для астрофотографии
    for (hour, weather) in forecast.hourly.iter().enumerate() {
        let mut hour_suitable = true;
        let mut hour_concerns = Vec::new();

        // Проверяем облачность (критично для астрофотографии)
        if weather.cloud_cover > 20.0 {
            hour_suitable = false;
            hour_concerns.push(format!(
                "Облачность {}% - не подходит для астрофотографии",
                weather.cloud_cover
            ));
        }

        // Проверяем видимость
        if weather.visibility < 10.0 {
            hour_suitable = false;
            hour_concerns.push(format!("Плохая видимость {} км", weather.visibility));
        }

        // Проверяем осадки
        if weather.precipitation_probability > 10.0 {
            hour_suitable = false;
            hour_concerns.push(format!(
                "Вероятность осадков {}%",
                weather.precipitation_probability
            ));
        }

        // Проверяем ветер (может влиять на качество снимков)
        if weather.wind_speed > 15.0 {
            hour_concerns.push(format!(
                "Сильный ветер {} м/с может влиять на качество",
                weather.wind_speed
            ));
        }

        // Ночные часы (22:00 - 4:00) лучше подходят для астрофотографии
        let is_night_hour = hour >= 22 || hour <= 4;

        if hour_suitable && is_night_hour {
            analysis.best_hours.push(hour);
        }

        if !hour_suitable {
            analysis.is_suitable = false;
            analysis.cloud_cover_issues.extend(hour_concerns);
        }
    }

    // Формируем рекомендации
    if analysis.is_suitable {
        analysis
            .recommendations
            .push("Отличные условия для астрофотографии!".to_string());
        analysis
            .recommendations
            .push("Ищите темные места вдали от городских огней".to_string());
        analysis
            .recommendations
            .push("Используйте штатив для длительных экспозиций".to_string());
    } else {
        analysis
            .recommendations
            .push("Условия не подходят для астрофотографии".to_string());
        analysis
            .recommendations
            .push("Рекомендуется перенести съемку на другой день".to_string());
    }

    // Проверяем общую облачность
    let avg_cloud_cover =
        forecast.hourly.iter().map(|w| w.cloud_cover).sum::<f64>() / forecast.hourly.len() as f64;
    if avg_cloud_cover > 50.0 {
        analysis.concerns.push(format!(
            "Высокая средняя облачность {}% - неблагоприятно для астрофотографии",
            avg_cloud_cover
        ));
    }

    analysis
}

#[derive(Debug)]
pub struct WeatherAnalysis {
    pub overall_score: f64,
    pub recommendations: Vec<String>,
    pub best_hours: Vec<usize>,
    pub concerns: Vec<String>,
}

#[derive(Debug)]
pub struct AstrophotographyAnalysis {
    pub is_suitable: bool,
    pub cloud_cover_issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub best_hours: Vec<usize>,
    pub concerns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Вспомогательные функции для создания тестовых данных
    fn create_test_weather_data() -> WeatherData {
        WeatherData {
            temperature: 20.0,
            humidity: 60.0,
            wind_speed: 5.0,
            cloud_cover: 30.0,
            visibility: 10.0,
            precipitation_probability: 5.0,
            description: "ясно".to_string(),
            timestamp: Utc::now(),
        }
    }

    fn create_test_forecast() -> WeatherForecast {
        let mut forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем 24 часа тестовых данных
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 15.0 + (hour as f64 * 0.5) - 6.0,
                humidity: 60.0 + (hour as f64 * 2.0) % 40.0,
                wind_speed: 5.0 + (hour as f64 * 0.3) % 15.0,
                cloud_cover: if !(6..=18).contains(&hour) {
                    20.0
                } else {
                    60.0
                },
                visibility: 10.0 - (hour as f64 * 0.1) % 5.0,
                precipitation_probability: if hour > 12 && hour < 18 { 30.0 } else { 5.0 },
                description: match hour {
                    6..=8 => "ясное утро".to_string(),
                    9..=11 => "солнечно".to_string(),
                    12..=14 => "переменная облачность".to_string(),
                    15..=17 => "облачно".to_string(),
                    18..=20 => "закат".to_string(),
                    _ => "ночь".to_string(),
                },
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            forecast.hourly.push(weather_data);
        }

        forecast
    }

    fn create_bad_weather_forecast() -> WeatherForecast {
        let mut forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем прогноз с плохими условиями
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: -5.0,               // Холодно
                humidity: 90.0,                  // Высокая влажность
                wind_speed: 25.0,                // Сильный ветер
                cloud_cover: 95.0,               // Высокая облачность
                visibility: 2.0,                 // Плохая видимость
                precipitation_probability: 80.0, // Высокая вероятность осадков
                description: "сильный дождь".to_string(),
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            forecast.hourly.push(weather_data);
        }

        forecast
    }

    #[test]
    fn test_weather_service_new() {
        let service = WeatherService::new("test_key".to_string(), "TestCity".to_string());

        assert_eq!(service.api_key, "test_key");
        assert_eq!(service.city, "TestCity");
        // demo_mode зависит от переменной окружения, поэтому не тестируем
    }

    #[test]
    fn test_weather_analysis_calculation() {
        let forecast = create_test_forecast();
        let analysis = analyze_weather_for_photography(&forecast);

        // Проверяем, что оценка в разумных пределах
        assert!(analysis.overall_score >= 0.0);
        assert!(analysis.overall_score <= 10.0);

        // Проверяем, что есть рекомендации
        assert!(!analysis.recommendations.is_empty());

        // Проверяем, что есть лучшие часы
        assert!(!analysis.best_hours.is_empty());
    }

    #[test]
    fn test_weather_analysis_bad_conditions() {
        let forecast = create_bad_weather_forecast();
        let analysis = analyze_weather_for_photography(&forecast);

        // При плохих условиях оценка должна быть низкой
        assert!(analysis.overall_score < 5.0);

        // Должны быть проблемы
        assert!(!analysis.concerns.is_empty());
    }

    #[test]
    fn test_astrophotography_analysis() {
        let forecast = create_test_forecast();
        let analysis = analyze_astrophotography_conditions(&forecast);

        // Проверяем структуру анализа
        assert!(!analysis.recommendations.is_empty());
        // Не проверяем на >= 0, это всегда true
    }

    #[test]
    fn test_astrophotography_analysis_bad_conditions() {
        let forecast = create_bad_weather_forecast();
        let analysis = analyze_astrophotography_conditions(&forecast);

        // При плохих условиях астрофотография должна быть непригодна
        assert!(!analysis.is_suitable);

        // Должны быть проблемы с облачностью
        assert!(!analysis.cloud_cover_issues.is_empty());
    }

    #[test]
    fn test_weather_data_validation() {
        let weather_data = create_test_weather_data();

        // Проверяем разумные пределы
        assert!(weather_data.temperature >= -50.0 && weather_data.temperature <= 60.0);
        assert!(weather_data.humidity >= 0.0 && weather_data.humidity <= 100.0);
        assert!(weather_data.wind_speed >= 0.0);
        assert!(weather_data.cloud_cover >= 0.0 && weather_data.cloud_cover <= 100.0);
        assert!(weather_data.visibility >= 0.0);
        assert!(
            weather_data.precipitation_probability >= 0.0
                && weather_data.precipitation_probability <= 100.0
        );
    }

    #[test]
    fn test_forecast_structure() {
        let forecast = create_test_forecast();

        // Проверяем, что прогноз содержит 24 часа
        assert_eq!(forecast.hourly.len(), 24);

        // Проверяем, что каждый час имеет валидные данные
        for (hour, weather) in forecast.hourly.iter().enumerate() {
            assert!(weather.temperature >= -50.0 && weather.temperature <= 60.0);
            assert!(weather.humidity >= 0.0 && weather.humidity <= 100.0);
            assert!(weather.wind_speed >= 0.0);
            assert!(weather.cloud_cover >= 0.0 && weather.cloud_cover <= 100.0);
            assert!(weather.visibility >= 0.0);
            assert!(
                weather.precipitation_probability >= 0.0
                    && weather.precipitation_probability <= 100.0
            );
            assert!(!weather.description.is_empty());

            // Проверяем, что timestamp увеличивается
            if hour > 0 {
                assert!(weather.timestamp > forecast.hourly[hour - 1].timestamp);
            }
        }
    }

    #[test]
    fn test_weather_analysis_edge_cases() {
        // Тест с пустым прогнозом
        let empty_forecast = WeatherForecast { hourly: Vec::new() };
        let analysis = analyze_weather_for_photography(&empty_forecast);

        // При пустом прогнозе оценка должна быть 0
        assert_eq!(analysis.overall_score, 0.0);
        assert!(analysis.best_hours.is_empty());
    }

    #[test]
    fn test_golden_hour_detection() {
        let forecast = create_test_forecast();
        let analysis = analyze_weather_for_photography(&forecast);

        // Проверяем, что золотые часы (6-8 и 18-20) имеют высокие оценки
        let golden_hours: Vec<usize> = vec![6, 7, 8, 18, 19, 20];

        // Проверяем, что хотя бы некоторые золотые часы попали в лучшие часы
        let has_golden_hours = golden_hours
            .iter()
            .any(|&hour| analysis.best_hours.contains(&hour));
        assert!(has_golden_hours || analysis.best_hours.is_empty());
    }

    #[test]
    fn test_weather_data_edge_cases() {
        // Тестируем граничные случаи для WeatherData
        let min_data = WeatherData {
            temperature: -50.0,             // Минимальная температура
            humidity: 0.0,                  // Минимальная влажность
            wind_speed: 0.0,                // Минимальная скорость ветра
            cloud_cover: 0.0,               // Минимальная облачность
            visibility: 0.0,                // Минимальная видимость
            precipitation_probability: 0.0, // Минимальная вероятность осадков
            description: "".to_string(),
            timestamp: Utc::now(),
        };

        let max_data = WeatherData {
            temperature: 60.0,                // Максимальная температура
            humidity: 100.0,                  // Максимальная влажность
            wind_speed: 100.0,                // Максимальная скорость ветра
            cloud_cover: 100.0,               // Максимальная облачность
            visibility: 50.0,                 // Максимальная видимость
            precipitation_probability: 100.0, // Максимальная вероятность осадков
            description: "экстремальные условия".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(min_data.temperature, -50.0);
        assert_eq!(max_data.temperature, 60.0);
        assert_eq!(min_data.humidity, 0.0);
        assert_eq!(max_data.humidity, 100.0);
    }

    #[test]
    fn test_weather_analysis_components() {
        let forecast = create_test_forecast();
        let analysis = analyze_weather_for_photography(&forecast);

        // Проверяем все компоненты анализа
        assert!(analysis.overall_score >= 0.0);
        assert!(analysis.overall_score <= 10.0);

        // Проверяем, что есть хотя бы одна рекомендация или проблема
        assert!(!analysis.recommendations.is_empty() || !analysis.concerns.is_empty());

        // Проверяем, что лучшие часы в разумных пределах
        for &hour in &analysis.best_hours {
            assert!((0..=23).contains(&hour));
        }
    }

    #[test]
    fn test_astrophotography_analysis_components() {
        let forecast = create_test_forecast();
        let analysis = analyze_astrophotography_conditions(&forecast);

        // Проверяем, что есть рекомендации
        assert!(!analysis.recommendations.is_empty());

        // Проверяем, что лучшие часы в разумных пределах
        for &hour in &analysis.best_hours {
            assert!((0..=23).contains(&hour));
        }
    }

    #[test]
    fn test_weather_service_demo_mode() {
        // Тестируем создание сервиса в demo режиме
        let service = WeatherService::new("demo_key".to_string(), "TestCity".to_string());

        // В demo режиме сервис должен работать без реальных API вызовов
        assert_eq!(service.city, "TestCity");
        assert_eq!(service.api_key, "demo_key");
    }

    #[test]
    fn test_weather_analysis_extreme_conditions() {
        // Тестируем анализ экстремальных погодных условий
        let mut extreme_forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем экстремальные условия
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: if hour < 12 { 50.0 } else { -30.0 }, // Экстремальные температуры
                humidity: if hour % 2 == 0 { 0.0 } else { 100.0 }, // Экстремальная влажность
                wind_speed: 50.0,                                  // Очень сильный ветер
                cloud_cover: if hour % 3 == 0 { 0.0 } else { 100.0 }, // Экстремальная облачность
                visibility: if hour % 4 == 0 { 0.1 } else { 50.0 }, // Экстремальная видимость
                precipitation_probability: if hour % 2 == 0 { 0.0 } else { 100.0 }, // Экстремальные осадки
                description: "экстремальные условия".to_string(),
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            extreme_forecast.hourly.push(weather_data);
        }

        let analysis = analyze_weather_for_photography(&extreme_forecast);

        // При экстремальных условиях оценка должна быть низкой
        assert!(analysis.overall_score < 5.0);

        // Должны быть проблемы
        assert!(!analysis.concerns.is_empty());
    }

    #[test]
    fn test_astrophotography_extreme_conditions() {
        // Тестируем анализ астрофотографии при экстремальных условиях
        let mut extreme_forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем условия с полной облачностью
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 20.0,
                humidity: 80.0,
                wind_speed: 10.0,
                cloud_cover: 100.0,              // Полная облачность
                visibility: 1.0,                 // Плохая видимость
                precipitation_probability: 90.0, // Высокая вероятность осадков
                description: "полная облачность".to_string(),
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            extreme_forecast.hourly.push(weather_data);
        }

        let analysis = analyze_astrophotography_conditions(&extreme_forecast);

        // При полной облачности астрофотография должна быть непригодна
        assert!(!analysis.is_suitable);

        // Должны быть проблемы с облачностью
        assert!(!analysis.cloud_cover_issues.is_empty());
    }

    #[test]
    fn test_weather_analysis_perfect_conditions() {
        // Тестируем анализ идеальных погодных условий
        let mut perfect_forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем идеальные условия
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 20.0,              // Комфортная температура
                humidity: 50.0,                 // Умеренная влажность
                wind_speed: 2.0,                // Легкий ветер
                cloud_cover: 10.0,              // Минимальная облачность
                visibility: 20.0,               // Отличная видимость
                precipitation_probability: 0.0, // Без осадков
                description: "идеальные условия".to_string(),
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            perfect_forecast.hourly.push(weather_data);
        }

        let analysis = analyze_weather_for_photography(&perfect_forecast);

        // При идеальных условиях оценка должна быть высокой
        assert!(analysis.overall_score >= 8.0);

        // Не должно быть проблем
        assert!(analysis.concerns.is_empty());
    }

    #[test]
    fn test_astrophotography_perfect_conditions() {
        // Тестируем анализ астрофотографии при идеальных условиях
        let mut perfect_forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем идеальные условия для астрофотографии
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 15.0,              // Прохладно
                humidity: 30.0,                 // Низкая влажность
                wind_speed: 1.0,                // Очень легкий ветер
                cloud_cover: 0.0,               // Без облаков
                visibility: 30.0,               // Отличная видимость
                precipitation_probability: 0.0, // Без осадков
                description: "идеальная ночь".to_string(),
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            perfect_forecast.hourly.push(weather_data);
        }

        let analysis = analyze_astrophotography_conditions(&perfect_forecast);

        // При идеальных условиях астрофотография должна быть пригодна
        assert!(analysis.is_suitable);

        // Не должно быть проблем с облачностью
        assert!(analysis.cloud_cover_issues.is_empty());
    }

    #[test]
    fn test_weather_analysis_single_hour() {
        // Тестируем анализ с одним часом данных
        let mut single_hour_forecast = WeatherForecast { hourly: Vec::new() };

        let weather_data = WeatherData {
            temperature: 20.0,
            humidity: 60.0,
            wind_speed: 5.0,
            cloud_cover: 30.0,
            visibility: 10.0,
            precipitation_probability: 5.0,
            description: "ясно".to_string(),
            timestamp: Utc::now(),
        };
        single_hour_forecast.hourly.push(weather_data);

        let analysis = analyze_weather_for_photography(&single_hour_forecast);

        // Проверяем, что анализ работает с одним часом
        assert!(analysis.overall_score >= 0.0);
        assert!(analysis.overall_score <= 10.0);
        assert_eq!(analysis.best_hours.len(), 1);
    }

    #[test]
    fn test_weather_analysis_mixed_conditions() {
        // Тестируем анализ смешанных условий
        let mut mixed_forecast = WeatherForecast { hourly: Vec::new() };

        // Создаем смешанные условия
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: if hour < 12 { 25.0 } else { 15.0 },
                humidity: if hour % 2 == 0 { 40.0 } else { 70.0 },
                wind_speed: if hour % 3 == 0 { 3.0 } else { 8.0 },
                cloud_cover: if !(6..=18).contains(&hour) {
                    20.0
                } else {
                    60.0
                },
                visibility: if hour % 4 == 0 { 5.0 } else { 15.0 },
                precipitation_probability: if hour > 10 && hour < 14 { 40.0 } else { 10.0 },
                description: "переменная погода".to_string(),
                timestamp: Utc::now() + chrono::Duration::hours(hour),
            };
            mixed_forecast.hourly.push(weather_data);
        }

        let analysis = analyze_weather_for_photography(&mixed_forecast);

        // Проверяем, что анализ работает со смешанными условиями
        assert!(analysis.overall_score >= 0.0);
        assert!(analysis.overall_score <= 10.0);
        assert!(!analysis.best_hours.is_empty());
    }
}
