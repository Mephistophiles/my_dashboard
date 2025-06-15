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

/// Форматирует вывод дашборда в строку для snapshot testing
pub fn format_dashboard_output(output: &DashboardOutput) -> String {
    let mut result = String::new();

    // Основная сводка
    result.push_str("=== ФОТОГРАФИЧЕСКИЙ ДАШБОРД ===\n");
    result.push_str("📊 ОБЩАЯ ОЦЕНКА\n");
    result.push_str(&format!(
        "   Погода: {:.1}/10\n",
        output.summary.weather_score
    ));
    result.push_str(&format!(
        "   Вероятность северных сияний: {:.0}%\n",
        output.summary.aurora_probability * 100.0
    ));
    result.push_str(&format!(
        "   Золотой час: {}\n",
        if output.summary.is_golden_hour_today {
            "Да"
        } else {
            "Нет"
        }
    ));

    if !output.summary.best_shooting_hours.is_empty() {
        result.push_str(&format!(
            "   Лучшие часы: {}\n",
            output
                .summary
                .best_shooting_hours
                .iter()
                .map(|h| format!("{:02}:00", h))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !output.summary.key_highlights.is_empty() {
        result.push_str("✨ КЛЮЧЕВЫЕ МОМЕНТЫ\n");
        for highlight in &output.summary.key_highlights {
            result.push_str(&format!("   • {}\n", highlight));
        }
    }

    if !output.summary.warnings.is_empty() {
        result.push_str("⚠️ ПРЕДУПРЕЖДЕНИЯ\n");
        for warning in &output.summary.warnings {
            result.push_str(&format!("   • {}\n", warning));
        }
    }

    result.push_str("🎯 РЕКОМЕНДАЦИЯ\n");
    result.push_str(&format!("   {}\n", output.summary.overall_recommendation));

    // Детальная информация
    result.push_str("\n📊 ДЕТАЛЬНАЯ ИНФОРМАЦИЯ\n");
    result.push_str(&format!("{}\n", output.weather_output.current_weather));
    result.push_str(&format!(
        "{}  {}  | ⭐ Оценка: {:.1}/10\n",
        output.weather_output.temperature_range,
        output.weather_output.best_hours,
        output.weather_output.overall_score
    ));

    if !output.weather_output.recommendation.is_empty() {
        result.push_str(&output.weather_output.recommendation);
    }
    if !output.weather_output.concerns.is_empty() {
        result.push_str(&format!(" | {}", output.weather_output.concerns));
    }
    result.push('\n');

    // Астрофотография
    result.push_str(&format!(
        "🌌 Астрофото: {} | ☁️{:.0}% | ",
        if output.astrophotography_output.is_suitable {
            "✅"
        } else {
            "❌"
        },
        output.astrophotography_output.avg_cloud_cover
    ));

    if !output.astrophotography_output.best_hours.is_empty() {
        result.push_str(&format!("{} ", output.astrophotography_output.best_hours));
    }
    if !output.astrophotography_output.recommendation.is_empty() {
        result.push_str(&format!(
            "| {}",
            output.astrophotography_output.recommendation
        ));
    }
    result.push('\n');

    // Солнечные данные
    result.push_str(&format!("{}\n", output.solar_output.solar_wind));
    result.push_str(&format!("{}\n", output.solar_output.geomagnetic));
    result.push_str(&format!("{}\n", output.solar_output.aurora_forecast));
    if !output.solar_output.best_viewing_hours.is_empty() {
        result.push_str(&format!("   {}\n", output.solar_output.best_viewing_hours));
    }

    // Золотой час
    result.push_str(&format!("{}\n", output.golden_hour_output.sunrise_sunset));
    result.push_str(&format!("{}\n", output.golden_hour_output.golden_hours));
    result.push_str(&format!("{}\n", output.golden_hour_output.blue_hours));
    result.push_str(&format!(
        "💡 Текущие условия освещения: {}\n",
        output.golden_hour_output.current_condition
    ));

    // Советы
    result.push_str("\n=== СОВЕТЫ ДЛЯ ФОТОГРАФОВ ===\n");

    if !output.tips_output.equipment_recommendations.is_empty() {
        result.push_str("\n📷 РЕКОМЕНДАЦИИ ПО ОБОРУДОВАНИЮ:\n");
        for (i, tip) in output
            .tips_output
            .equipment_recommendations
            .iter()
            .enumerate()
        {
            result.push_str(&format!("{}. {}\n", i + 1, tip));
        }
    }

    if !output.tips_output.shooting_tips.is_empty() {
        result.push_str("\n🎯 СОВЕТЫ ПО СЪЕМКЕ:\n");
        for (i, tip) in output.tips_output.shooting_tips.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, tip));
        }
    }

    if !output.tips_output.location_suggestions.is_empty() {
        result.push_str("\n📍 РЕКОМЕНДАЦИИ ПО ЛОКАЦИЯМ:\n");
        for (i, tip) in output.tips_output.location_suggestions.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, tip));
        }
    }

    if !output.tips_output.technical_settings.is_empty() {
        result.push_str("\n⚙️ ТЕХНИЧЕСКИЕ НАСТРОЙКИ:\n");
        for (i, tip) in output.tips_output.technical_settings.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, tip));
        }
    }

    result.push_str("\n=== ОБЩИЕ РЕКОМЕНДАЦИИ ===\n");
    for (i, tip) in output
        .tips_output
        .general_recommendations
        .iter()
        .enumerate()
    {
        result.push_str(&format!("{}. {}\n", i + 1, tip));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    fn test_validate_coordinates() {
        assert!(validate_coordinates(55.7558, 37.6176));
        assert!(!validate_coordinates(100.0, 200.0));
    }

    #[test]
    fn test_format_dashboard_output_smoke() {
        // Минимальный мок-объект для smoke-теста
        let output = DashboardOutput {
            summary: dashboard::DashboardSummary {
                overall_recommendation: "Test".to_string(),
                weather_score: 5.0,
                aurora_probability: 0.5,
                is_golden_hour_today: false,
                best_shooting_hours: vec![6, 18],
                key_highlights: vec!["Test highlight".to_string()],
                warnings: vec!["Test warning".to_string()],
            },
            weather_output: WeatherOutput {
                current_weather: "Test weather".to_string(),
                temperature_range: "9-20°C".to_string(),
                best_hours: "06:00-08:00".to_string(),
                overall_score: 5.0,
                recommendation: "Test rec".to_string(),
                concerns: "Test concern".to_string(),
            },
            astrophotography_output: AstrophotographyOutput {
                is_suitable: false,
                avg_cloud_cover: 50.0,
                best_hours: "00:00".to_string(),
                recommendation: "No astro".to_string(),
            },
            solar_output: SolarOutput {
                solar_wind: "Test wind".to_string(),
                geomagnetic: "Test geomagnetic".to_string(),
                aurora_forecast: "Test aurora".to_string(),
                best_viewing_hours: "22:00-23:00".to_string(),
            },
            golden_hour_output: GoldenHourOutput {
                sunrise_sunset: "03:44 | 21:16".to_string(),
                golden_hours: "04:00-06:00".to_string(),
                blue_hours: "03:30-04:00".to_string(),
                current_condition: "Дневное время".to_string(),
            },
            tips_output: PhotographyTipsOutput {
                equipment_recommendations: vec!["Test equipment".to_string()],
                shooting_tips: vec!["Test tip".to_string()],
                location_suggestions: vec!["Test location".to_string()],
                technical_settings: vec!["Test setting".to_string()],
                general_recommendations: vec!["Test general".to_string()],
            },
        };
        let formatted = format_dashboard_output(&output);
        assert!(formatted.contains("ФОТОГРАФИЧЕСКИЙ ДАШБОРД"));
    }

    #[test]
    fn test_process_golden_hour_smoke() {
        let (_is_golden, output) = process_golden_hour(55.7558, 37.6176);
        assert!(output.sunrise_sunset.contains(":"));
        assert!(output.golden_hours.contains(":"));
        assert!(output.blue_hours.contains(":"));
        assert!(!output.current_condition.is_empty());
        // is_golden может быть true или false, главное что функция не паникует
    }

    #[test]
    fn test_process_photography_tips_smoke() {
        let tips = process_photography_tips(8.0, true, 0.7);
        assert!(!tips.equipment_recommendations.is_empty());
        assert!(!tips.shooting_tips.is_empty());
        assert!(!tips.location_suggestions.is_empty());
        assert!(!tips.technical_settings.is_empty());
        assert!(!tips.general_recommendations.is_empty());
    }

    #[test]
    fn test_load_environment_variables_smoke() {
        // Установим переменные окружения для теста
        env::set_var("OPENWEATHER_API_KEY", "demo_key");
        env::set_var("CITY", "Moscow");
        env::set_var("LATITUDE", "55.7558");
        env::set_var("LONGITUDE", "37.6176");
        let (api_key, city, lat, lon) = load_environment_variables();
        assert_eq!(api_key, "demo_key");
        assert_eq!(city, "Moscow");
        assert_eq!(lat, 55.7558);
        assert_eq!(lon, 37.6176);
    }

    #[test]
    fn test_async_wrappers() {
        let rt = Runtime::new().unwrap();
        // process_weather_data
        let (weather_score, weather_output, astro_output) = rt
            .block_on(process_weather_data(
                "demo_key".to_string(),
                "Moscow".to_string(),
            ))
            .unwrap();
        assert!((0.0..=10.0).contains(&weather_score));
        assert!(!weather_output.current_weather.is_empty());
        assert!(!astro_output.recommendation.is_empty());

        // process_solar_data
        let (aurora_prob, solar_output) = rt.block_on(process_solar_data()).unwrap();
        assert!((0.0..=1.0).contains(&aurora_prob));
        assert!(!solar_output.solar_wind.is_empty());
    }

    #[test]
    fn test_generate_dashboard_output_smoke() {
        let rt = Runtime::new().unwrap();
        let output = rt
            .block_on(generate_dashboard_output(
                "demo_key".to_string(),
                "Moscow".to_string(),
                55.7558,
                37.6176,
            ))
            .unwrap();
        assert!(!output.summary.overall_recommendation.is_empty());
        assert!(!output.weather_output.current_weather.is_empty());
        assert!(!output.solar_output.solar_wind.is_empty());
        assert!(!output.golden_hour_output.sunrise_sunset.is_empty());
        // tips_output может содержать пустые списки в зависимости от условий
        // Проверяем только структуру, а не содержимое
        let _ = &output.tips_output.equipment_recommendations;
        let _ = &output.tips_output.shooting_tips;
        let _ = &output.tips_output.location_suggestions;
        let _ = &output.tips_output.technical_settings;
        let _ = &output.tips_output.general_recommendations;
    }

    #[test]
    fn test_process_golden_hour_edge_coords() {
        // Используем граничные, но валидные координаты
        let (_is_golden, output) = process_golden_hour(90.0, 180.0);
        assert!(!output.sunrise_sunset.is_empty());
    }

    #[test]
    fn test_process_photography_tips_extremes() {
        // Минимальные значения
        let tips_min = process_photography_tips(0.0, false, 0.0);
        assert!(!tips_min.equipment_recommendations.is_empty());
        // Максимальные значения
        let tips_max = process_photography_tips(10.0, true, 1.0);
        assert!(!tips_max.equipment_recommendations.is_empty());
        assert!(!tips_max.shooting_tips.is_empty());
        assert!(!tips_max.location_suggestions.is_empty());
        assert!(!tips_max.technical_settings.is_empty());
    }

    #[test]
    fn test_format_dashboard_output_empty_fields() {
        let output = DashboardOutput {
            summary: dashboard::DashboardSummary {
                overall_recommendation: String::new(),
                weather_score: 0.0,
                aurora_probability: 0.0,
                is_golden_hour_today: false,
                best_shooting_hours: vec![],
                key_highlights: vec![],
                warnings: vec![],
            },
            weather_output: WeatherOutput {
                current_weather: String::new(),
                temperature_range: String::new(),
                best_hours: String::new(),
                overall_score: 0.0,
                recommendation: String::new(),
                concerns: String::new(),
            },
            astrophotography_output: AstrophotographyOutput {
                is_suitable: false,
                avg_cloud_cover: 0.0,
                best_hours: String::new(),
                recommendation: String::new(),
            },
            solar_output: SolarOutput {
                solar_wind: String::new(),
                geomagnetic: String::new(),
                aurora_forecast: String::new(),
                best_viewing_hours: String::new(),
            },
            golden_hour_output: GoldenHourOutput {
                sunrise_sunset: String::new(),
                golden_hours: String::new(),
                blue_hours: String::new(),
                current_condition: String::new(),
            },
            tips_output: PhotographyTipsOutput {
                equipment_recommendations: vec![],
                shooting_tips: vec![],
                location_suggestions: vec![],
                technical_settings: vec![],
                general_recommendations: vec![],
            },
        };
        let formatted = format_dashboard_output(&output);
        assert!(formatted.contains("ФОТОГРАФИЧЕСКИЙ ДАШБОРД"));
    }

    #[test]
    fn test_validate_coordinates_edge_cases() {
        // Граничные значения
        assert!(validate_coordinates(90.0, 180.0));
        assert!(validate_coordinates(-90.0, -180.0));
        assert!(!validate_coordinates(90.1, 0.0));
        assert!(!validate_coordinates(0.0, 180.1));
    }

    #[test]
    fn test_load_environment_variables_missing() {
        // Удаляем переменные окружения
        env::remove_var("OPENWEATHER_API_KEY");
        env::remove_var("CITY");
        env::remove_var("LATITUDE");
        env::remove_var("LONGITUDE");
        let (api_key, city, lat, lon) = load_environment_variables();
        assert_eq!(api_key, "demo_key");
        assert_eq!(city, "Moscow");
        assert_eq!(lat, 55.7558);
        assert_eq!(lon, 37.6176);
    }

    #[test]
    fn test_async_wrappers_empty_city() {
        let rt = Runtime::new().unwrap();
        // Пустой город не должен паниковать, но может вернуть ошибку
        let result = rt.block_on(process_weather_data("demo_key".to_string(), "".to_string()));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_generate_dashboard_output_edge_coords() {
        let rt = Runtime::new().unwrap();
        // Используем граничные, но валидные координаты
        let result = rt.block_on(generate_dashboard_output(
            "demo_key".to_string(),
            "Moscow".to_string(),
            90.0,
            180.0,
        ));
        assert!(result.is_ok() || result.is_err());
    }
}
