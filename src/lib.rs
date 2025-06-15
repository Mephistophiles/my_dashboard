//! # My Dashboard - Дашборд для фотографов
//!
//! Библиотека для создания персонализированного дашборда, который помогает фотографам
//! планировать съемки на основе погодных условий, золотого часа и активности северных сияний.
//!
//! ## Основные возможности
//!
//! - **Анализ погоды**: Оценка условий для фотографии на основе температуры, ветра, облачности и видимости
//! - **Золотой час**: Расчет оптимального времени для съемки с мягким освещением
//! - **Северные сияния**: Прогноз активности северных сияний для астрофотографии
//! - **Персонализированные советы**: Рекомендации по оборудованию и настройкам камеры
//!
//! ## Пример использования
//!
//! ```rust
//! use my_dashboard::dashboard::PhotographyDashboard;
//!
//! // Создаем дашборд
//! let dashboard = PhotographyDashboard::new(
//!     "your_api_key".to_string(),
//!     "Moscow".to_string(),
//!     55.7558,
//!     37.6176,
//! );
//!
//! // Для асинхронного использования:
//! // #[tokio::main]
//! // async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! //     let summary = dashboard.generate_dashboard().await?;
//! //     dashboard.print_dashboard(&summary);
//! //     Ok(())
//! // }
//! ```
//!
//! ## Модули
//!
//! - [`dashboard`] - Основной дашборд и сводка
//! - [`weather`] - Анализ погодных условий
//! - [`golden_hour`] - Расчет золотого часа
//! - [`solar`] - Прогноз северных сияний
//! - [`photography_tips`] - Советы для фотографов

pub mod dashboard;
pub mod golden_hour;
pub mod photography_tips;
pub mod solar;
pub mod weather;

use anyhow::Result;
use std::env;

// Структуры для хранения строк вместо принтов
#[derive(Debug, Clone)]
pub struct WeatherOutput {
    pub current_weather: String,
    pub temperature_range: String,
    pub best_hours: String,
    pub overall_score: f64,
    pub recommendation: String,
    pub concerns: String,
}

#[derive(Debug, Clone)]
pub struct AstrophotographyOutput {
    pub is_suitable: bool,
    pub avg_cloud_cover: f64,
    pub best_hours: String,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct SolarOutput {
    pub solar_wind: String,
    pub geomagnetic: String,
    pub aurora_forecast: String,
    pub best_viewing_hours: String,
}

#[derive(Debug, Clone)]
pub struct GoldenHourOutput {
    pub sunrise_sunset: String,
    pub golden_hours: String,
    pub blue_hours: String,
    pub current_condition: String,
}

#[derive(Debug, Clone)]
pub struct PhotographyTipsOutput {
    pub equipment_recommendations: Vec<String>,
    pub shooting_tips: Vec<String>,
    pub location_suggestions: Vec<String>,
    pub technical_settings: Vec<String>,
    pub general_recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DashboardOutput {
    pub summary: dashboard::DashboardSummary,
    pub weather_output: WeatherOutput,
    pub astrophotography_output: AstrophotographyOutput,
    pub solar_output: SolarOutput,
    pub golden_hour_output: GoldenHourOutput,
    pub tips_output: PhotographyTipsOutput,
}

// Функции для обработки бизнес-логики
pub async fn process_weather_data(
    api_key: String,
    city: String,
) -> Result<(f64, WeatherOutput, AstrophotographyOutput)> {
    let weather_service = weather::WeatherService::new(api_key, city);
    let forecast = weather_service.get_weather_forecast().await?;

    let analysis = weather::analyze_weather_for_photography(&forecast);
    let weather_score = analysis.overall_score;

    let weather_output = generate_weather_output(&forecast, &analysis);
    let astrophotography_output = generate_astrophotography_output(&forecast);

    Ok((weather_score, weather_output, astrophotography_output))
}

pub async fn process_solar_data() -> Result<(f64, SolarOutput)> {
    let solar_output = generate_solar_output().await?;
    let aurora_probability = solar_output
        .aurora_forecast
        .split_whitespace()
        .find(|s| s.ends_with('%'))
        .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
        .map(|p| p / 100.0)
        .unwrap_or(0.0);

    Ok((aurora_probability, solar_output))
}

pub fn process_golden_hour(latitude: f64, longitude: f64) -> (bool, GoldenHourOutput) {
    let golden_hour_service = golden_hour::GoldenHourService::new(latitude, longitude);
    let is_golden_hour = golden_hour_service.is_golden_hour();
    let golden_hour_output = generate_golden_hour_output(&golden_hour_service);

    (is_golden_hour, golden_hour_output)
}

pub fn process_photography_tips(
    weather_score: f64,
    is_golden_hour: bool,
    aurora_probability: f64,
) -> PhotographyTipsOutput {
    let tips_service = photography_tips::PhotographyTipsService::new();
    let personalized_tips =
        tips_service.get_tips_for_weather(weather_score, is_golden_hour, aurora_probability);
    let general_tips = tips_service.get_general_recommendations();

    PhotographyTipsOutput {
        equipment_recommendations: personalized_tips.equipment_recommendations,
        shooting_tips: personalized_tips.shooting_tips,
        location_suggestions: personalized_tips.location_suggestions,
        technical_settings: personalized_tips.technical_settings,
        general_recommendations: general_tips,
    }
}

pub async fn generate_dashboard_output(
    api_key: String,
    city: String,
    latitude: f64,
    longitude: f64,
) -> Result<DashboardOutput, anyhow::Error> {
    // Создаем дашборд
    let dashboard =
        dashboard::PhotographyDashboard::new(api_key.clone(), city.clone(), latitude, longitude);
    let summary = dashboard.generate_dashboard().await?;

    // Обрабатываем данные погоды
    let (weather_score, weather_output, astrophotography_output) =
        process_weather_data(api_key.clone(), city.clone()).await?;

    // Обрабатываем солнечные данные
    let (aurora_probability, solar_output) = process_solar_data().await?;

    // Обрабатываем золотой час
    let (is_golden_hour, golden_hour_output) = process_golden_hour(latitude, longitude);

    // Обрабатываем советы
    let tips_output = process_photography_tips(weather_score, is_golden_hour, aurora_probability);

    Ok(DashboardOutput {
        summary,
        weather_output,
        astrophotography_output,
        solar_output,
        golden_hour_output,
        tips_output,
    })
}

// Вспомогательные функции для генерации строк
fn generate_weather_output(
    forecast: &weather::WeatherForecast,
    analysis: &weather::WeatherAnalysis,
) -> WeatherOutput {
    let current_weather = if let Some(current) = forecast.hourly.first() {
        format!(
            "🌤️ Погода: 🌡️{:.1}°C  ☁️{:.0}%  💨{:.1}м/с  🌧️{:.0}%  📝{}",
            current.temperature,
            current.cloud_cover,
            current.wind_speed,
            current.precipitation_probability,
            current.description
        )
    } else {
        "Нет данных о погоде".to_string()
    };

    let min_temp = forecast
        .hourly
        .iter()
        .map(|w| w.temperature)
        .fold(f64::INFINITY, f64::min);
    let max_temp = forecast
        .hourly
        .iter()
        .map(|w| w.temperature)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_precip = forecast
        .hourly
        .iter()
        .map(|w| w.precipitation_probability)
        .fold(0.0, f64::max);
    let max_wind = forecast
        .hourly
        .iter()
        .map(|w| w.wind_speed)
        .fold(0.0, f64::max);

    let temperature_range = format!(
        "📊 Диапазон: 🌡️{}-{}°C  💨Ветер до {:.1}м/с  🌧️Осадки до {:.0}%",
        min_temp as i32, max_temp as i32, max_wind, max_precip
    );

    let best_hours = if !analysis.best_hours.is_empty() {
        let mut intervals = Vec::new();
        let mut start = analysis.best_hours[0];
        let mut end = start;

        for &hour in &analysis.best_hours[1..] {
            if hour == end + 1 {
                end = hour;
            } else {
                if start == end {
                    intervals.push(format!("{:02}:00", start));
                } else {
                    intervals.push(format!("{:02}:00-{:02}:00", start, end));
                }
                start = hour;
                end = hour;
            }
        }
        if start == end {
            intervals.push(format!("{:02}:00", start));
        } else {
            intervals.push(format!("{:02}:00-{:02}:00", start, end));
        }

        format!(
            "🕐 Лучшие часы: {}",
            intervals
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        "Нет подходящих часов".to_string()
    };

    let recommendation = if !analysis.recommendations.is_empty() {
        format!("💡 Рекомендация: {}", analysis.recommendations[0])
    } else {
        String::new()
    };

    let concerns = if !analysis.concerns.is_empty() {
        format!("⚠️ Проблемы: {}", analysis.concerns[0])
    } else {
        String::new()
    };

    WeatherOutput {
        current_weather,
        temperature_range,
        best_hours,
        overall_score: analysis.overall_score,
        recommendation,
        concerns,
    }
}

fn generate_astrophotography_output(forecast: &weather::WeatherForecast) -> AstrophotographyOutput {
    let analysis = weather::analyze_astrophotography_conditions(forecast);
    let avg_cloud_cover =
        forecast.hourly.iter().map(|w| w.cloud_cover).sum::<f64>() / forecast.hourly.len() as f64;

    let best_hours = if !analysis.best_hours.is_empty() {
        let mut intervals = Vec::new();
        let mut start = analysis.best_hours[0];
        let mut end = start;

        for &hour in &analysis.best_hours[1..] {
            if hour == end + 1 {
                end = hour;
            } else {
                if start == end {
                    intervals.push(format!("{:02}:00", start));
                } else {
                    intervals.push(format!("{:02}:00-{:02}:00", start, end));
                }
                start = hour;
                end = hour;
            }
        }
        if start == end {
            intervals.push(format!("{:02}:00", start));
        } else {
            intervals.push(format!("{:02}:00-{:02}:00", start, end));
        }

        format!(
            "🕐 Лучшие часы: {}",
            intervals
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        "Нет подходящих часов".to_string()
    };

    let recommendation = if !analysis.recommendations.is_empty() {
        format!("💡 {}", analysis.recommendations[0])
    } else {
        String::new()
    };

    AstrophotographyOutput {
        is_suitable: analysis.is_suitable,
        avg_cloud_cover,
        best_hours,
        recommendation,
    }
}

async fn generate_solar_output() -> Result<SolarOutput> {
    let solar_wind = match solar::fetch_solar_wind_data().await {
        Ok(data) => format!(
            "🌞 Солнечный ветер: 💨{:.1}км/с  📊{:.1}частиц/см³  🌡️{:.0}K  🕐{}",
            data.speed,
            data.density,
            data.temperature,
            data.timestamp.format("%H:%M")
        ),
        Err(e) => format!("❌ Ошибка получения данных солнечного ветра: {}", e),
    };

    let geomagnetic = match solar::fetch_geomagnetic_data().await {
        Ok(data) => format!(
            "🌍 Геомагнитные данные: 🧲Kp {:.1}  🌌Активность сияний {:.1}/10  🕐{}",
            data.kp_index,
            data.aurora_activity,
            data.timestamp.format("%H:%M")
        ),
        Err(e) => format!("❌ Ошибка получения геомагнитных данных: {}", e),
    };

    let (aurora_forecast, best_viewing_hours) = match solar::predict_aurora().await {
        Ok(forecast) => {
            let forecast_str = format!(
                "🌌 Прогноз северных сияний: {}%  📊{}  💡{}",
                (forecast.visibility_probability * 100.0) as i32,
                forecast.intensity_level,
                forecast.conditions
            );

            let hours_str = if !forecast.best_viewing_hours.is_empty() {
                let mut intervals = Vec::new();
                let mut start = forecast.best_viewing_hours[0];
                let mut end = start;

                for &hour in &forecast.best_viewing_hours[1..] {
                    if hour == end + 1 {
                        end = hour;
                    } else {
                        if start == end {
                            intervals.push(format!("{:02}:00", start));
                        } else {
                            intervals.push(format!("{:02}:00-{:02}:00", start, end));
                        }
                        start = hour;
                        end = hour;
                    }
                }
                if start == end {
                    intervals.push(format!("{:02}:00", start));
                } else {
                    intervals.push(format!("{:02}:00-{:02}:00", start, end));
                }

                format!("🕐 Лучшие часы для наблюдения: {}", intervals.join(", "))
            } else {
                String::new()
            };

            (forecast_str, hours_str)
        }
        Err(e) => (
            format!("❌ Ошибка прогноза северных сияний: {}", e),
            String::new(),
        ),
    };

    Ok(SolarOutput {
        solar_wind,
        geomagnetic,
        aurora_forecast,
        best_viewing_hours,
    })
}

fn generate_golden_hour_output(service: &golden_hour::GoldenHourService) -> GoldenHourOutput {
    let current_time = chrono::Local::now();
    let info = service.calculate_golden_hours(current_time);
    let current_condition = service.get_current_lighting_condition(current_time);

    let sunrise_sunset = format!(
        "🌅 Восход: {} | 🌆 Закат: {}",
        info.sunrise.format("%H:%M"),
        info.sunset.format("%H:%M")
    );

    let golden_hours = format!(
        "🌅 Золотой час утро: {}-{} | 🌆 Золотой час вечер: {}-{}",
        info.golden_hour_morning_start.format("%H:%M"),
        info.golden_hour_morning_end.format("%H:%M"),
        info.golden_hour_evening_start.format("%H:%M"),
        info.golden_hour_evening_end.format("%H:%M")
    );

    let blue_hours = format!(
        "🌅 Синий час утро: {}-{} | 🌆 Синий час вечер: {}-{}",
        info.blue_hour_morning_start.format("%H:%M"),
        info.blue_hour_morning_end.format("%H:%M"),
        info.blue_hour_evening_start.format("%H:%M"),
        info.blue_hour_evening_end.format("%H:%M")
    );

    GoldenHourOutput {
        sunrise_sunset,
        golden_hours,
        blue_hours,
        current_condition,
    }
}

// Функции для загрузки и валидации переменных окружения
pub fn load_environment_variables() -> (String, String, f64, f64) {
    let api_key = env::var("OPENWEATHER_API_KEY").unwrap_or_else(|_| {
        log::warn!("OPENWEATHER_API_KEY не найден, используем demo_key");
        "demo_key".to_string()
    });

    let city = env::var("CITY").unwrap_or_else(|_| {
        log::info!("CITY не найден, используем Москва");
        "Moscow".to_string()
    });

    let latitude = env::var("LATITUDE")
        .unwrap_or_else(|_| "55.7558".to_string())
        .parse::<f64>()
        .unwrap_or(55.7558);

    let longitude = env::var("LONGITUDE")
        .unwrap_or_else(|_| "37.6176".to_string())
        .parse::<f64>()
        .unwrap_or(37.6176);

    (api_key, city, latitude, longitude)
}

pub fn validate_coordinates(latitude: f64, longitude: f64) -> bool {
    (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)
}
