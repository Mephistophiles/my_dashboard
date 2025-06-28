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
//! use my_dashboard::solar::predict_aurora;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Получаем прогноз северных сияний
//!     let forecast = predict_aurora().await?;
//!     println!("Вероятность северных сияний: {:.1}%",
//!         forecast.visibility_probability * 100.0);
//!     println!("Скорость солнечного ветра: {} км/с", forecast.solar_wind.speed);
//!     println!("Kp индекс: {}", forecast.geomagnetic.kp_index);
//!
//!     Ok(())
//! }
//! ```

use crate::{get_current_utc_time, is_demo_mode};
use anyhow::Result;
use chrono::{DateTime, Utc};
use log::debug;
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
    /// Информация о солнечном ветре
    pub solar_wind: SolarWindData,
    /// Информация о геомагнитной активности
    pub geomagnetic: GeomagneticData,
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
async fn fetch_solar_wind_data() -> Result<SolarWindData> {
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

    debug!("🌞 API ЗАПРОС: NOAA SWEPAM API (солнечный ветер)");
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

async fn fetch_geomagnetic_data() -> Result<GeomagneticData> {
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

    debug!("🌍 API ЗАПРОС: NOAA Planetary K-index API (геомагнитные данные)");
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
        solar_wind,
        geomagnetic,
        visibility_probability: probability,
        intensity_level,
        best_viewing_hours: best_hours,
        conditions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
}
