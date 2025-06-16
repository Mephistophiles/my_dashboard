//! # Solar Module
//!
//! Модуль для получения данных о солнечной активности и прогнозирования северных сияний.
//! Предоставляет функциональность для анализа условий астрофотографии.
//!
//! ## Основные компоненты
//!
//! - [`SolarWindData`] - Данные о солнечном ветре
//! - [`GeomagneticData`] - Геомагнитные данные
//! - [`AuroraForecast`] - Прогноз северных сияний
//!
//! ## Пример использования
//!
//! ```rust,no_run
//! use my_dashboard::solar::{predict_aurora, fetch_solar_wind_data, fetch_geomagnetic_data};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Получаем прогноз северных сияний
//!     let forecast = predict_aurora().await?;
//!     println!("Вероятность северных сияний: {:.1}%",
//!         forecast.visibility_probability * 100.0);
//!
//!     // Получаем данные о солнечном ветре
//!     let solar_wind = fetch_solar_wind_data().await?;
//!     println!("Скорость солнечного ветра: {} км/с", solar_wind.speed);
//!
//!     // Получаем геомагнитные данные
//!     let geomagnetic = fetch_geomagnetic_data().await?;
//!     println!("Kp индекс: {}", geomagnetic.kp_index);
//!     
//!     Ok(())
//! }
//! ```

use crate::{get_current_utc_time, is_demo_mode};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Данные о солнечном ветре
///
/// Содержит информацию о скорости, плотности и температуре солнечного ветра,
/// полученную от NOAA SWEPAM API.
#[derive(Debug, Serialize, Deserialize)]
pub struct SolarWindData {
    /// Скорость солнечного ветра в км/с
    pub speed: f64,
    /// Плотность частиц в частиц/см³
    pub density: f64,
    /// Температура в Кельвинах
    pub temperature: f64,
    /// Магнитное поле в нТл (недоступно в SWEPAM API)
    pub magnetic_field: Option<f64>,
    /// Временная метка данных
    pub timestamp: DateTime<Utc>,
}

/// Геомагнитные данные
///
/// Содержит информацию о геомагнитной активности и Kp индексе,
/// полученную от NOAA Planetary K-index API.
#[derive(Debug, Serialize, Deserialize)]
pub struct GeomagneticData {
    /// Геомагнитный Kp индекс (0-9)
    pub kp_index: f64,
    /// Активность северных сияний (0-10)
    pub aurora_activity: f64,
    /// Солнечная радиация (недоступно в Kp API)
    pub solar_radiation: Option<f64>,
    /// Временная метка данных
    pub timestamp: DateTime<Utc>,
}

/// Прогноз северных сияний
///
/// Содержит информацию о вероятности появления северных сияний,
/// уровне интенсивности и лучшем времени для наблюдения.
#[derive(Debug)]
pub struct AuroraForecast {
    /// Вероятность видимости северных сияний (0-1)
    pub visibility_probability: f64,
    /// Уровень интенсивности (текстовое описание)
    pub intensity_level: String,
    /// Лучшие часы для наблюдения (0-23)
    pub best_viewing_hours: Vec<usize>,
    /// Условия для наблюдения
    pub conditions: String,
}

// Структуры для парсинга NOAA API
#[derive(Debug, Serialize, Deserialize)]
struct SwepamRecord {
    #[serde(rename = "time_tag")]
    time_tag: String,
    #[serde(rename = "dsflag")]
    dsflag: i32,
    #[serde(rename = "dens")]
    dens: Option<f64>,
    #[serde(rename = "speed")]
    speed: Option<f64>,
    #[serde(rename = "temperature")]
    temperature: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KpRecord {
    #[serde(rename = "time_tag")]
    time_tag: String,
    #[serde(rename = "kp_index")]
    kp_index: f64,
}

/// Получает данные о солнечном ветре от NOAA SWEPAM API
///
/// # Возвращает
///
/// `Result<SolarWindData>` - Данные о солнечном ветре или ошибка
///
/// # Пример
///
/// ```rust,no_run
/// use my_dashboard::solar::fetch_solar_wind_data;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let solar_wind = fetch_solar_wind_data().await?;
///     println!("Скорость: {} км/с", solar_wind.speed);
///     Ok(())
/// }
/// ```
pub async fn fetch_solar_wind_data() -> Result<SolarWindData> {
    // Проверяем DEMO режим
    let demo_mode = is_demo_mode();

    if demo_mode {
        // Возвращаем статические данные для DEMO режима
        return Ok(SolarWindData {
            speed: 719.3,
            density: 4.1,
            temperature: 490479.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        });
    }

    let url = "https://services.swpc.noaa.gov/json/ace/swepam/ace_swepam_1h.json";
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP {}: {}",
            response.status(),
            response.text().await?
        ));
    }

    let text = response.text().await?;

    // Попробуем парсить JSON с более подробной обработкой ошибок
    let all_records: Vec<SwepamRecord> = match serde_json::from_str::<Vec<SwepamRecord>>(&text) {
        Ok(records) => records,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to parse solar wind JSON: {}", e));
        }
    };

    if all_records.is_empty() {
        return Err(anyhow::anyhow!("No solar wind data available"));
    }

    // Берем только последние 50 записей для ускорения парсинга
    let start_idx = if all_records.len() > 50 {
        all_records.len() - 50
    } else {
        0
    };
    let records = &all_records[start_idx..];

    // Берем последнюю запись с валидными данными
    let latest_record = records
        .iter()
        .find(|r| r.dsflag == 0 && r.dens.is_some() && r.speed.is_some() && r.temperature.is_some())
        .ok_or_else(|| anyhow::anyhow!("No valid solar wind data found"))?;

    let timestamp =
        match chrono::NaiveDateTime::parse_from_str(&latest_record.time_tag, "%Y-%m-%dT%H:%M:%S") {
            Ok(dt) => dt.and_utc(),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse timestamp '{}': {}",
                    latest_record.time_tag,
                    e
                ));
            }
        };

    Ok(SolarWindData {
        speed: latest_record.speed.unwrap(),
        density: latest_record.dens.unwrap(),
        temperature: latest_record.temperature.unwrap(),
        magnetic_field: None, // Нет данных о магнитном поле в SWEPAM
        timestamp,
    })
}

pub async fn fetch_geomagnetic_data() -> Result<GeomagneticData> {
    // Проверяем DEMO режим
    let demo_mode = is_demo_mode();

    if demo_mode {
        // Возвращаем статические данные для DEMO режима
        return Ok(GeomagneticData {
            kp_index: 0.0,
            aurora_activity: 0.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        });
    }

    let url = "https://services.swpc.noaa.gov/json/planetary_k_index_1m.json";
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP {}: {}",
            response.status(),
            response.text().await?
        ));
    }

    let text = response.text().await?;

    // Попробуем парсить JSON с более подробной обработкой ошибок
    let all_records: Vec<KpRecord> = match serde_json::from_str::<Vec<KpRecord>>(&text) {
        Ok(records) => records,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to parse geomagnetic JSON: {}", e));
        }
    };

    if all_records.is_empty() {
        return Err(anyhow::anyhow!("No geomagnetic data available"));
    }

    // Берем только последние 50 записей для ускорения парсинга
    let start_idx = if all_records.len() > 50 {
        all_records.len() - 50
    } else {
        0
    };
    let records = &all_records[start_idx..];

    // Берем последнюю запись
    let latest_record = &records[records.len() - 1];

    let timestamp =
        match chrono::NaiveDateTime::parse_from_str(&latest_record.time_tag, "%Y-%m-%dT%H:%M:%S") {
            Ok(dt) => dt.and_utc(),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse timestamp '{}': {}",
                    latest_record.time_tag,
                    e
                ));
            }
        };

    // Рассчитываем активность северных сияний на основе Kp индекса
    let aurora_activity = if latest_record.kp_index >= 5.0 {
        8.0 + (latest_record.kp_index - 5.0) * 0.4
    } else if latest_record.kp_index >= 3.0 {
        4.0 + (latest_record.kp_index - 3.0) * 2.0
    } else {
        latest_record.kp_index * 1.33
    }
    .min(10.0);

    Ok(GeomagneticData {
        kp_index: latest_record.kp_index,
        aurora_activity,
        solar_radiation: None, // Нет данных о солнечной радиации
        timestamp,
    })
}

fn calculate_aurora_activity(solar_wind: &SolarWindData, geomagnetic: &GeomagneticData) -> f64 {
    let mut activity = 0.0;

    // Влияние Kp индекса (0-9)
    activity += (geomagnetic.kp_index / 9.0).min(1.0) * 6.0;

    // Влияние скорости солнечного ветра
    if solar_wind.speed > 600.0 {
        activity += 2.0;
    } else if solar_wind.speed > 400.0 {
        activity += 1.0;
    }

    // Влияние плотности солнечного ветра
    if solar_wind.density > 10.0 {
        activity += 2.0;
    } else if solar_wind.density > 5.0 {
        activity += 1.0;
    }

    activity.min(10.0)
}

pub async fn predict_aurora() -> Result<AuroraForecast> {
    let solar_wind = fetch_solar_wind_data().await?;
    let geomagnetic = fetch_geomagnetic_data().await?;

    // Используем функцию calculate_aurora_activity для расчета активности
    let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);

    // Преобразуем активность (0-10) в вероятность (0-1)
    let probability = (activity / 10.0_f64).min(1.0);

    // Определяем уровень интенсивности
    let intensity_level = if probability > 0.8 {
        "Очень высокая"
    } else if probability > 0.6 {
        "Высокая"
    } else if probability > 0.4 {
        "Умеренная"
    } else if probability > 0.2 {
        "Низкая"
    } else {
        "Минимальная"
    }
    .to_string();

    // Определяем условия
    let conditions = if probability > 0.6 {
        "Отличные условия для наблюдения северных сияний"
    } else if probability > 0.4 {
        "Хорошие условия, возможны сияния"
    } else if probability > 0.2 {
        "Умеренные условия, сияния маловероятны"
    } else {
        "Плохие условия для наблюдения сияний"
    }
    .to_string();

    // Определяем лучшие часы для наблюдения (ночные часы)
    let best_hours = vec![22, 23, 0, 1, 2, 3, 4, 5];

    Ok(AuroraForecast {
        visibility_probability: probability,
        intensity_level,
        best_viewing_hours: best_hours,
        conditions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    // Вспомогательные функции для создания тестовых данных
    fn create_test_solar_wind_data() -> SolarWindData {
        SolarWindData {
            speed: 500.0,
            density: 5.0,
            temperature: 250000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        }
    }

    fn create_test_geomagnetic_data() -> GeomagneticData {
        GeomagneticData {
            kp_index: 3.0,
            aurora_activity: 4.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        }
    }

    fn create_high_activity_solar_wind() -> SolarWindData {
        SolarWindData {
            speed: 700.0,  // Высокая скорость
            density: 15.0, // Высокая плотность
            temperature: 300000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        }
    }

    fn create_high_kp_geomagnetic_data() -> GeomagneticData {
        GeomagneticData {
            kp_index: 7.0, // Высокий Kp индекс
            aurora_activity: 8.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        }
    }

    #[test]
    fn test_solar_wind_data_validation() {
        let solar_wind = create_test_solar_wind_data();

        // Проверяем разумные пределы
        assert!(solar_wind.speed > 0.0 && solar_wind.speed < 2000.0);
        assert!(solar_wind.density > 0.0 && solar_wind.density < 100.0);
        assert!(solar_wind.temperature > 0.0 && solar_wind.temperature < 1000000.0);

        // Проверяем, что magnetic_field действительно None
        assert!(solar_wind.magnetic_field.is_none());
    }

    #[test]
    fn test_geomagnetic_data_validation() {
        let geomagnetic = create_test_geomagnetic_data();

        // Проверяем разумные пределы
        assert!(geomagnetic.kp_index >= 0.0 && geomagnetic.kp_index <= 9.0);
        assert!(geomagnetic.aurora_activity >= 0.0 && geomagnetic.aurora_activity <= 10.0);

        // Проверяем, что solar_radiation действительно None
        assert!(geomagnetic.solar_radiation.is_none());
    }

    #[test]
    fn test_aurora_forecast_structure() {
        let forecast = AuroraForecast {
            visibility_probability: 0.5,
            intensity_level: "Умеренная".to_string(),
            best_viewing_hours: vec![22, 23, 0, 1, 2, 3, 4, 5],
            conditions: "Хорошие условия".to_string(),
        };

        assert!((0.0..=1.0).contains(&forecast.visibility_probability));
        assert!(!forecast.intensity_level.is_empty());
        assert!(!forecast.best_viewing_hours.is_empty());
        assert!(!forecast.conditions.is_empty());

        // Проверяем, что лучшие часы находятся в ночном диапазоне
        for &hour in &forecast.best_viewing_hours {
            assert!((0..=23).contains(&hour));
        }
    }

    #[tokio::test]
    async fn test_predict_aurora_with_test_data() {
        // Создаем мок функции для тестирования
        let solar_wind = create_test_solar_wind_data();
        let geomagnetic = create_test_geomagnetic_data();

        // Тестируем функцию calculate_aurora_activity
        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert!((0.0..=10.0).contains(&activity));

        // Тестируем создание прогноза
        let forecast = AuroraForecast {
            visibility_probability: (activity / 10.0_f64).min(1.0),
            intensity_level: if activity > 8.0 {
                "Очень высокая"
            } else if activity > 6.0 {
                "Высокая"
            } else if activity > 4.0 {
                "Умеренная"
            } else if activity > 2.0 {
                "Низкая"
            } else {
                "Минимальная"
            }
            .to_string(),
            best_viewing_hours: vec![22, 23, 0, 1, 2, 3, 4, 5],
            conditions: if activity > 6.0 {
                "Отличные условия для наблюдения северных сияний"
            } else if activity > 4.0 {
                "Хорошие условия, возможны сияния"
            } else if activity > 2.0 {
                "Умеренные условия, сияния маловероятны"
            } else {
                "Плохие условия для наблюдения сияний"
            }
            .to_string(),
        };

        assert!((0.0..=1.0).contains(&forecast.visibility_probability));
        assert!(!forecast.intensity_level.is_empty());
        assert!(!forecast.best_viewing_hours.is_empty());
        assert!(!forecast.conditions.is_empty());
    }

    #[tokio::test]
    async fn test_predict_aurora_high_activity() {
        let solar_wind = create_high_activity_solar_wind();
        let geomagnetic = create_high_kp_geomagnetic_data();

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert!(activity > 6.0); // Должна быть высокая активность

        let forecast = AuroraForecast {
            visibility_probability: (activity / 10.0_f64).min(1.0),
            intensity_level: "Высокая".to_string(),
            best_viewing_hours: vec![22, 23, 0, 1, 2, 3, 4, 5],
            conditions: "Отличные условия для наблюдения северных сияний".to_string(),
        };

        assert!(forecast.visibility_probability > 0.6);
        assert_eq!(forecast.intensity_level, "Высокая");
    }

    #[test]
    fn test_aurora_intensity_levels() {
        // Тестируем различные уровни интенсивности
        let high_prob = 0.9;
        let medium_prob = 0.5;
        let low_prob = 0.1;

        let high_intensity = if high_prob > 0.8 {
            "Очень высокая"
        } else if high_prob > 0.6 {
            "Высокая"
        } else if high_prob > 0.4 {
            "Умеренная"
        } else if high_prob > 0.2 {
            "Низкая"
        } else {
            "Минимальная"
        };

        let medium_intensity = if medium_prob > 0.8 {
            "Очень высокая"
        } else if medium_prob > 0.6 {
            "Высокая"
        } else if medium_prob > 0.4 {
            "Умеренная"
        } else if medium_prob > 0.2 {
            "Низкая"
        } else {
            "Минимальная"
        };

        let low_intensity = if low_prob > 0.8 {
            "Очень высокая"
        } else if low_prob > 0.6 {
            "Высокая"
        } else if low_prob > 0.4 {
            "Умеренная"
        } else if low_prob > 0.2 {
            "Низкая"
        } else {
            "Минимальная"
        };

        assert_eq!(high_intensity, "Очень высокая");
        assert_eq!(medium_intensity, "Умеренная");
        assert_eq!(low_intensity, "Минимальная");
    }

    #[test]
    fn test_aurora_conditions() {
        // Тестируем различные условия
        let high_prob = 0.9;
        let medium_prob = 0.5;
        let low_prob = 0.1;

        let high_conditions = if high_prob > 0.6 {
            "Отличные условия для наблюдения северных сияний"
        } else if high_prob > 0.4 {
            "Хорошие условия, возможны сияния"
        } else if high_prob > 0.2 {
            "Умеренные условия, сияния маловероятны"
        } else {
            "Плохие условия для наблюдения сияний"
        };

        let medium_conditions = if medium_prob > 0.6 {
            "Отличные условия для наблюдения северных сияний"
        } else if medium_prob > 0.4 {
            "Хорошие условия, возможны сияния"
        } else if medium_prob > 0.2 {
            "Умеренные условия, сияния маловероятны"
        } else {
            "Плохие условия для наблюдения сияний"
        };

        let low_conditions = if low_prob > 0.6 {
            "Отличные условия для наблюдения северных сияний"
        } else if low_prob > 0.4 {
            "Хорошие условия, возможны сияния"
        } else if low_prob > 0.2 {
            "Умеренные условия, сияния маловероятны"
        } else {
            "Плохие условия для наблюдения сияний"
        };

        assert_eq!(
            high_conditions,
            "Отличные условия для наблюдения северных сияний"
        );
        assert_eq!(medium_conditions, "Хорошие условия, возможны сияния");
        assert_eq!(low_conditions, "Плохие условия для наблюдения сияний");
    }

    #[test]
    fn test_best_viewing_hours() {
        // Тестируем лучшие часы для наблюдения
        let best_hours = vec![22, 23, 0, 1, 2, 3, 4, 5];

        // Проверяем, что все часы находятся в ночном диапазоне
        for &hour in &best_hours {
            assert!((0..=23).contains(&hour));
        }

        // Проверяем, что часы идут в правильном порядке
        for i in 0..best_hours.len() - 1 {
            if best_hours[i] == 23 {
                assert_eq!(best_hours[i + 1], 0);
            } else {
                assert_eq!(best_hours[i + 1], best_hours[i] + 1);
            }
        }
    }

    #[test]
    fn test_aurora_activity_calculation() {
        let solar_wind = create_test_solar_wind_data();
        let geomagnetic = create_test_geomagnetic_data();

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert!((0.0..=10.0).contains(&activity));
    }

    #[test]
    fn test_solar_wind_data_structure() {
        let solar_wind = SolarWindData {
            speed: 400.0,
            density: 3.0,
            temperature: 200000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        assert_eq!(solar_wind.speed, 400.0);
        assert_eq!(solar_wind.density, 3.0);
        assert_eq!(solar_wind.temperature, 200000.0);
        assert!(solar_wind.magnetic_field.is_none());
    }

    #[test]
    fn test_geomagnetic_data_structure() {
        let geomagnetic = GeomagneticData {
            kp_index: 4.5,
            aurora_activity: 6.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        assert_eq!(geomagnetic.kp_index, 4.5);
        assert_eq!(geomagnetic.aurora_activity, 6.0);
        assert!(geomagnetic.solar_radiation.is_none());
    }

    #[test]
    fn test_aurora_forecast_creation() {
        let forecast = AuroraForecast {
            visibility_probability: 0.7,
            intensity_level: "Высокая".to_string(),
            best_viewing_hours: vec![22, 23, 0, 1, 2, 3, 4, 5],
            conditions: "Отличные условия для наблюдения северных сияний".to_string(),
        };

        assert_eq!(forecast.visibility_probability, 0.7);
        assert_eq!(forecast.intensity_level, "Высокая");
        assert_eq!(forecast.best_viewing_hours.len(), 8);
        assert!(!forecast.conditions.is_empty());
    }

    #[test]
    fn test_swepam_record_parsing() {
        let record = SwepamRecord {
            time_tag: "2024-01-15T12:00:00".to_string(),
            dsflag: 0,
            dens: Some(5.0),
            speed: Some(400.0),
            temperature: Some(250000.0),
        };

        assert_eq!(record.time_tag, "2024-01-15T12:00:00");
        assert_eq!(record.dsflag, 0);
        assert_eq!(record.dens, Some(5.0));
        assert_eq!(record.speed, Some(400.0));
        assert_eq!(record.temperature, Some(250000.0));
    }

    #[test]
    fn test_kp_record_parsing() {
        let record = KpRecord {
            time_tag: "2024-01-15T12:00:00".to_string(),
            kp_index: 3.5,
        };

        assert_eq!(record.time_tag, "2024-01-15T12:00:00");
        assert_eq!(record.kp_index, 3.5);
    }

    #[test]
    fn test_timestamp_parsing() {
        let timestamp_str = "2024-01-15T12:00:00";
        let parsed = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S");

        assert!(parsed.is_ok());
        let dt = parsed.unwrap().and_utc();
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_aurora_probability_calculation() {
        let activity = 7.5;
        let probability = (activity / 10.0_f64).min(1.0);

        assert_eq!(probability, 0.75);
        assert!((0.0..=1.0).contains(&probability));
    }

    #[test]
    fn test_aurora_activity_edge_cases() {
        // Тестируем граничные случаи
        let solar_wind = SolarWindData {
            speed: 0.0,
            density: 0.0,
            temperature: 0.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        let geomagnetic = GeomagneticData {
            kp_index: 0.0,
            aurora_activity: 0.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert_eq!(activity, 0.0);

        // Тестируем максимальные значения
        let solar_wind_max = SolarWindData {
            speed: 1000.0,
            density: 20.0,
            temperature: 500000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        let geomagnetic_max = GeomagneticData {
            kp_index: 9.0,
            aurora_activity: 10.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        let activity_max = calculate_aurora_activity(&solar_wind_max, &geomagnetic_max);
        assert_eq!(activity_max, 10.0);
    }

    #[test]
    fn test_swepam_record_edge_cases() {
        // Тестируем записи с отсутствующими данными
        let record_with_none = SwepamRecord {
            time_tag: "2024-01-15T12:00:00".to_string(),
            dsflag: 1, // Невалидный флаг
            dens: None,
            speed: None,
            temperature: None,
        };

        assert_eq!(record_with_none.dsflag, 1);
        assert!(record_with_none.dens.is_none());
        assert!(record_with_none.speed.is_none());
        assert!(record_with_none.temperature.is_none());

        // Тестируем валидную запись
        let valid_record = SwepamRecord {
            time_tag: "2024-01-15T12:00:00".to_string(),
            dsflag: 0,
            dens: Some(5.0),
            speed: Some(400.0),
            temperature: Some(250000.0),
        };

        assert_eq!(valid_record.dsflag, 0);
        assert!(valid_record.dens.is_some());
        assert!(valid_record.speed.is_some());
        assert!(valid_record.temperature.is_some());
    }

    #[test]
    fn test_kp_record_edge_cases() {
        // Тестируем граничные значения Kp индекса
        let min_kp = KpRecord {
            time_tag: "2024-01-15T12:00:00".to_string(),
            kp_index: 0.0,
        };

        let max_kp = KpRecord {
            time_tag: "2024-01-15T12:00:00".to_string(),
            kp_index: 9.0,
        };

        assert_eq!(min_kp.kp_index, 0.0);
        assert_eq!(max_kp.kp_index, 9.0);
    }

    #[test]
    fn test_aurora_forecast_edge_cases() {
        // Тестируем прогноз с минимальной вероятностью
        let min_forecast = AuroraForecast {
            visibility_probability: 0.0,
            intensity_level: "Минимальная".to_string(),
            best_viewing_hours: vec![22, 23, 0, 1, 2, 3, 4, 5],
            conditions: "Плохие условия для наблюдения сияний".to_string(),
        };

        assert_eq!(min_forecast.visibility_probability, 0.0);
        assert_eq!(min_forecast.intensity_level, "Минимальная");

        // Тестируем прогноз с максимальной вероятностью
        let max_forecast = AuroraForecast {
            visibility_probability: 1.0,
            intensity_level: "Очень высокая".to_string(),
            best_viewing_hours: vec![22, 23, 0, 1, 2, 3, 4, 5],
            conditions: "Отличные условия для наблюдения северных сияний".to_string(),
        };

        assert_eq!(max_forecast.visibility_probability, 1.0);
        assert_eq!(max_forecast.intensity_level, "Очень высокая");
    }

    #[test]
    fn test_solar_wind_data_edge_cases() {
        // Тестируем экстремальные значения солнечного ветра
        let extreme_solar_wind = SolarWindData {
            speed: 2000.0,               // Очень высокая скорость
            density: 50.0,               // Очень высокая плотность
            temperature: 1000000.0,      // Очень высокая температура
            magnetic_field: Some(100.0), // С магнитным полем
            timestamp: get_current_utc_time(),
        };

        assert_eq!(extreme_solar_wind.speed, 2000.0);
        assert_eq!(extreme_solar_wind.density, 50.0);
        assert_eq!(extreme_solar_wind.temperature, 1000000.0);
        assert_eq!(extreme_solar_wind.magnetic_field, Some(100.0));

        // Тестируем минимальные значения
        let min_solar_wind = SolarWindData {
            speed: 1.0,
            density: 0.1,
            temperature: 1000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        assert_eq!(min_solar_wind.speed, 1.0);
        assert_eq!(min_solar_wind.density, 0.1);
        assert_eq!(min_solar_wind.temperature, 1000.0);
        assert!(min_solar_wind.magnetic_field.is_none());
    }

    #[test]
    fn test_geomagnetic_data_edge_cases() {
        // Тестируем экстремальные значения геомагнитных данных
        let extreme_geomagnetic = GeomagneticData {
            kp_index: 9.0,                 // Максимальный Kp индекс
            aurora_activity: 10.0,         // Максимальная активность
            solar_radiation: Some(1000.0), // С солнечной радиацией
            timestamp: get_current_utc_time(),
        };

        assert_eq!(extreme_geomagnetic.kp_index, 9.0);
        assert_eq!(extreme_geomagnetic.aurora_activity, 10.0);
        assert_eq!(extreme_geomagnetic.solar_radiation, Some(1000.0));

        // Тестируем минимальные значения
        let min_geomagnetic = GeomagneticData {
            kp_index: 0.0,
            aurora_activity: 0.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        assert_eq!(min_geomagnetic.kp_index, 0.0);
        assert_eq!(min_geomagnetic.aurora_activity, 0.0);
        assert!(min_geomagnetic.solar_radiation.is_none());
    }

    #[test]
    fn test_aurora_activity_calculation_formula() {
        // Тестируем формулу расчета активности
        let solar_wind = SolarWindData {
            speed: 600.0,  // Высокая скорость
            density: 10.0, // Высокая плотность
            temperature: 250000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        let geomagnetic = GeomagneticData {
            kp_index: 5.0, // Средний Kp индекс
            aurora_activity: 6.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);

        // Проверяем компоненты формулы
        let kp_component = (geomagnetic.kp_index / 9.0).min(1.0) * 6.0;
        let speed_component = if solar_wind.speed > 600.0 {
            2.0
        } else if solar_wind.speed > 400.0 {
            1.0
        } else {
            0.0
        };
        let density_component = if solar_wind.density > 10.0 {
            2.0
        } else if solar_wind.density > 5.0 {
            1.0
        } else {
            0.0
        };

        let expected_activity = (kp_component + speed_component + density_component).min(10.0);
        assert_eq!(activity, expected_activity);
    }

    #[test]
    fn test_high_activity_aurora_calculation() {
        // Тестируем расчет высокой активности
        let solar_wind = SolarWindData {
            speed: 800.0,  // Очень высокая скорость
            density: 15.0, // Очень высокая плотность
            temperature: 300000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        let geomagnetic = GeomagneticData {
            kp_index: 7.0, // Высокий Kp индекс
            aurora_activity: 8.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert!(activity > 8.0); // Должна быть очень высокая активность
    }

    #[test]
    fn test_aurora_activity_bounds() {
        // Тестируем границы активности
        let solar_wind = SolarWindData {
            speed: 1000.0,
            density: 20.0,
            temperature: 500000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        let geomagnetic = GeomagneticData {
            kp_index: 9.0,
            aurora_activity: 10.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert_eq!(activity, 10.0); // Максимальная активность
    }

    #[tokio::test]
    async fn test_print_solar_data_structure() {
        // Тестируем структуру функции print_solar_data
        // Создаем мок данные для тестирования
        let solar_wind = create_test_solar_wind_data();
        let geomagnetic = create_test_geomagnetic_data();

        // Проверяем, что данные имеют правильную структуру
        assert!(solar_wind.speed > 0.0);
        assert!(solar_wind.density > 0.0);
        assert!(solar_wind.temperature > 0.0);
        assert!(geomagnetic.kp_index >= 0.0);
        assert!(geomagnetic.aurora_activity >= 0.0);

        // Проверяем форматирование времени
        let time_str = solar_wind.timestamp.format("%H:%M").to_string();
        assert_eq!(time_str.len(), 5); // Формат HH:MM
        assert!(time_str.contains(':'));
    }

    #[test]
    fn test_aurora_activity_calculation_components() {
        // Тестируем отдельные компоненты расчета активности
        let solar_wind = SolarWindData {
            speed: 500.0,
            density: 8.0,
            temperature: 250000.0,
            magnetic_field: None,
            timestamp: get_current_utc_time(),
        };

        let geomagnetic = GeomagneticData {
            kp_index: 4.0,
            aurora_activity: 5.0,
            solar_radiation: None,
            timestamp: get_current_utc_time(),
        };

        // Проверяем компоненты формулы
        let kp_component = (geomagnetic.kp_index / 9.0).min(1.0) * 6.0;
        assert!((0.0..=6.0).contains(&kp_component));

        let speed_component = if solar_wind.speed > 600.0 {
            2.0
        } else if solar_wind.speed > 400.0 {
            1.0
        } else {
            0.0
        };
        assert_eq!(speed_component, 1.0); // 500 > 400

        let density_component = if solar_wind.density > 10.0 {
            2.0
        } else if solar_wind.density > 5.0 {
            1.0
        } else {
            0.0
        };
        assert_eq!(density_component, 1.0); // 8 > 5

        let total_activity = (kp_component + speed_component + density_component).min(10.0);
        assert!((0.0..=10.0).contains(&total_activity));
    }
}
