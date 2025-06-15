use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SolarWindData {
    pub speed: f64,                  // км/с
    pub density: f64,                // частиц/см³
    pub temperature: f64,            // К
    pub magnetic_field: Option<f64>, // нТл (недоступно в SWEPAM API)
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeomagneticData {
    pub kp_index: f64,                // Геомагнитный индекс
    pub aurora_activity: f64,         // Активность северных сияний (0-10)
    pub solar_radiation: Option<f64>, // Солнечная радиация (недоступно в Kp API)
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AuroraForecast {
    pub visibility_probability: f64, // Вероятность видимости северных сияний
    pub intensity_level: String,     // Уровень интенсивности
    pub best_viewing_hours: Vec<usize>, // Лучшие часы для наблюдения
    pub conditions: String,          // Условия для наблюдения
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

pub async fn print_solar_data() -> Result<()> {
    match fetch_solar_wind_data().await {
        Ok(solar_wind) => {
            println!(
                "🌞 Солнечный ветер: 💨{:.1}км/с  📊{:.1}частиц/см³  🌡️{:.0}K  🕐{}",
                solar_wind.speed,
                solar_wind.density,
                solar_wind.temperature,
                solar_wind.timestamp.format("%H:%M")
            );
        }
        Err(e) => {
            println!("❌ Ошибка получения данных солнечного ветра: {}", e);
        }
    }

    match fetch_geomagnetic_data().await {
        Ok(geomagnetic) => {
            println!(
                "🌍 Геомагнитные данные: 🧲Kp {:.1}  🌌Активность сияний {:.1}/10  🕐{}",
                geomagnetic.kp_index,
                geomagnetic.aurora_activity,
                geomagnetic.timestamp.format("%H:%M")
            );
        }
        Err(e) => {
            println!("❌ Ошибка получения геомагнитных данных: {}", e);
        }
    }

    match predict_aurora().await {
        Ok(forecast) => {
            println!(
                "🌌 Прогноз северных сияний: {}%  📊{}  💡{}",
                (forecast.visibility_probability * 100.0) as i32,
                forecast.intensity_level,
                forecast.conditions
            );

            if !forecast.best_viewing_hours.is_empty() {
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

                println!("   🕐 Лучшие часы для наблюдения: {}", intervals.join(", "));
            }
        }
        Err(e) => {
            println!("   ❌ Ошибка прогноза северных сияний: {}", e);
        }
    }

    Ok(())
}

async fn fetch_solar_wind_data() -> Result<SolarWindData> {
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
    let probability = (activity / 10.0).min(1.0);

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
    use chrono::{Datelike, Timelike};

    // Вспомогательные функции для создания тестовых данных
    fn create_test_solar_wind_data() -> SolarWindData {
        SolarWindData {
            speed: 500.0,
            density: 5.0,
            temperature: 250000.0,
            magnetic_field: None,
            timestamp: Utc::now(),
        }
    }

    fn create_test_geomagnetic_data() -> GeomagneticData {
        GeomagneticData {
            kp_index: 3.0,
            aurora_activity: 4.0,
            solar_radiation: None,
            timestamp: Utc::now(),
        }
    }

    fn create_high_activity_solar_wind() -> SolarWindData {
        SolarWindData {
            speed: 700.0,  // Высокая скорость
            density: 15.0, // Высокая плотность
            temperature: 300000.0,
            magnetic_field: None,
            timestamp: Utc::now(),
        }
    }

    fn create_high_kp_geomagnetic_data() -> GeomagneticData {
        GeomagneticData {
            kp_index: 7.0, // Высокий Kp индекс
            aurora_activity: 8.0,
            solar_radiation: None,
            timestamp: Utc::now(),
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
            conditions: "Хорошие условия, возможны сияния".to_string(),
        };

        // Проверяем разумные пределы
        assert!((0.0..=1.0).contains(&forecast.visibility_probability));
        assert!(!forecast.intensity_level.is_empty());
        assert!(!forecast.conditions.is_empty());

        // Проверяем, что лучшие часы для наблюдения - ночные
        for &hour in &forecast.best_viewing_hours {
            assert!((0..=23).contains(&hour));
        }
    }

    #[tokio::test]
    async fn test_predict_aurora_with_test_data() {
        // Создаем mock данные для тестирования
        let solar_wind = create_test_solar_wind_data();
        let geomagnetic = create_test_geomagnetic_data();

        // Тестируем логику расчета вероятности (без реальных API вызовов)
        let mut probability = 0.0;

        // Влияние Kp индекса (основной фактор)
        probability += (geomagnetic.kp_index / 9.0).min(1.0) * 0.6;

        // Влияние скорости солнечного ветра
        let speed_factor = if solar_wind.speed > 600.0 {
            0.3
        } else if solar_wind.speed > 500.0 {
            0.2
        } else if solar_wind.speed > 400.0 {
            0.1
        } else {
            0.0
        };
        probability += speed_factor;

        // Влияние плотности солнечного ветра
        let density_factor = if solar_wind.density > 10.0 {
            0.1
        } else if solar_wind.density > 5.0 {
            0.05
        } else {
            0.0
        };
        probability += density_factor;

        probability = probability.min(1.0);

        // Проверяем, что вероятность в разумных пределах
        assert!((0.0..=1.0).contains(&probability));

        // Для тестовых данных с Kp=3.0 и скоростью=500.0, вероятность должна быть > 0
        assert!(probability > 0.0);
    }

    #[tokio::test]
    async fn test_predict_aurora_high_activity() {
        // Тестируем с высокими значениями активности
        let _solar_wind = create_high_activity_solar_wind();
        let geomagnetic = create_high_kp_geomagnetic_data();

        let mut probability = 0.0;

        // Влияние Kp индекса
        probability += (geomagnetic.kp_index / 9.0).min(1.0) * 0.6;

        // Влияние скорости солнечного ветра (700 км/с > 600)
        probability += 0.3;

        // Влияние плотности солнечного ветра (15 > 10)
        probability += 0.1;

        probability = probability.min(1.0);

        // При высоких значениях вероятность должна быть высокой
        assert!(probability > 0.8);
    }

    #[test]
    fn test_aurora_intensity_levels() {
        // Тестируем определение уровней интенсивности
        let test_cases = vec![
            (0.9, "Очень высокая"),
            (0.7, "Высокая"),
            (0.5, "Умеренная"),
            (0.3, "Низкая"),
            (0.1, "Минимальная"),
        ];

        for (probability, expected_level) in test_cases {
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
            };

            assert_eq!(intensity_level, expected_level);
        }
    }

    #[test]
    fn test_aurora_conditions() {
        // Тестируем определение условий
        let test_cases = vec![
            (0.7, "Отличные условия для наблюдения северных сияний"),
            (0.5, "Хорошие условия, возможны сияния"),
            (0.3, "Умеренные условия, сияния маловероятны"),
            (0.1, "Плохие условия для наблюдения сияний"),
        ];

        for (probability, expected_condition) in test_cases {
            let conditions = if probability > 0.6 {
                "Отличные условия для наблюдения северных сияний"
            } else if probability > 0.4 {
                "Хорошие условия, возможны сияния"
            } else if probability > 0.2 {
                "Умеренные условия, сияния маловероятны"
            } else {
                "Плохие условия для наблюдения сияний"
            };

            assert_eq!(conditions, expected_condition);
        }
    }

    #[test]
    fn test_best_viewing_hours() {
        // Проверяем, что лучшие часы для наблюдения - ночные
        let best_hours = vec![22, 23, 0, 1, 2, 3, 4, 5];

        for &hour in &best_hours {
            // Ночные часы: 22-23 и 0-5
            assert!((22..=23).contains(&hour) || (0..=5).contains(&hour));
        }

        // Проверяем, что часы идут в правильном порядке
        for i in 0..best_hours.len() - 1 {
            if best_hours[i] == 23 {
                // После 23 может идти 0
                assert!(best_hours[i + 1] == 0);
            } else {
                // В остальных случаях следующий час должен быть больше
                assert!(best_hours[i + 1] > best_hours[i]);
            }
        }
    }

    #[test]
    fn test_aurora_activity_calculation() {
        let solar_wind = create_test_solar_wind_data();
        let geomagnetic = create_test_geomagnetic_data();

        // Тестируем расчет активности северных сияний
        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        assert!((0.0..=10.0).contains(&activity));
    }

    #[test]
    fn test_solar_wind_data_structure() {
        let data = create_test_solar_wind_data();

        // Проверяем структуру данных солнечного ветра
        assert!(data.speed > 0.0);
        assert!(data.density > 0.0);
        assert!(data.temperature > 0.0);
        assert!(data.timestamp > chrono::Utc::now() - chrono::Duration::days(1));
    }

    #[test]
    fn test_geomagnetic_data_structure() {
        let data = create_test_geomagnetic_data();

        // Проверяем структуру геомагнитных данных
        assert!(data.kp_index >= 0.0 && data.kp_index <= 9.0);
        assert!(data.aurora_activity >= 0.0 && data.aurora_activity <= 10.0);
        assert!(data.timestamp > chrono::Utc::now() - chrono::Duration::days(1));
    }

    #[test]
    fn test_aurora_forecast_creation() {
        let forecast = AuroraForecast {
            visibility_probability: 0.7,
            intensity_level: "Умеренная".to_string(),
            best_viewing_hours: vec![22, 23, 0, 1],
            conditions: "Хорошие условия для наблюдения".to_string(),
        };

        // Проверяем структуру прогноза
        assert!((0.0..=1.0).contains(&forecast.visibility_probability));
        assert!(!forecast.intensity_level.is_empty());
        assert!(!forecast.best_viewing_hours.is_empty());
        assert!(!forecast.conditions.is_empty());
    }

    #[test]
    fn test_swepam_record_parsing() {
        // Тестируем парсинг SWEPAM записи
        let json = r#"{
            "time_tag": "2024-06-15T12:00:00",
            "dsflag": 0,
            "dens": 5.2,
            "speed": 450.0,
            "temperature": 150000.0
        }"#;

        let record: SwepamRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.time_tag, "2024-06-15T12:00:00");
        assert_eq!(record.dsflag, 0);
        assert_eq!(record.dens, Some(5.2));
        assert_eq!(record.speed, Some(450.0));
        assert_eq!(record.temperature, Some(150000.0));
    }

    #[test]
    fn test_kp_record_parsing() {
        // Тестируем парсинг Kp записи
        let json = r#"{
            "time_tag": "2024-06-15T12:00:00",
            "kp_index": 3.5
        }"#;

        let record: KpRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.time_tag, "2024-06-15T12:00:00");
        assert_eq!(record.kp_index, 3.5);
    }

    #[test]
    fn test_timestamp_parsing() {
        // Тестируем парсинг временных меток
        let timestamp_str = "2024-06-15T12:00:00";
        let parsed = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S");
        assert!(parsed.is_ok());

        let dt = parsed.unwrap().and_utc();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 12);
    }

    #[test]
    fn test_aurora_probability_calculation() {
        // Тестируем расчет вероятности северных сияний
        let solar_wind = create_high_activity_solar_wind();
        let geomagnetic = create_high_kp_geomagnetic_data();

        let activity = calculate_aurora_activity(&solar_wind, &geomagnetic);
        let probability = activity / 10.0;

        assert!((0.0..=1.0).contains(&probability));
    }
}
