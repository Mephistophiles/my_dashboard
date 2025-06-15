use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SolarWindData {
    pub speed: f64,          // км/с
    pub density: f64,        // частиц/см³
    pub temperature: f64,    // К
    pub magnetic_field: f64, // нТл
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeomagneticData {
    pub kp_index: f64,        // Геомагнитный индекс
    pub aurora_activity: f64, // Активность северных сияний (0-10)
    pub solar_radiation: f64, // Солнечная радиация
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
    println!("{}", "☀️ Solar Wind:".bold().yellow());
    
    // Получаем данные солнечного ветра
    match fetch_solar_wind_data().await {
        Ok(data) => {
            println!("   Скорость: {:.1} км/s", data.speed);
            println!("   Плотность: {:.1} частиц/см³", data.density);
            println!("   Температура: {:.0} K", data.temperature);
            println!("   Магнитное поле: {:.1} нТл", data.magnetic_field);
            println!("   Время: {}", data.timestamp.format("%H:%M UTC"));
        }
        Err(e) => {
            println!("   Error - {}", e);
        }
    }

    println!("{}", "🌍 Geomagnetic:".bold().blue());
    
    // Получаем геомагнитные данные
    match fetch_geomagnetic_data().await {
        Ok(data) => {
            println!("   Kp индекс: {:.1}", data.kp_index);
            println!("   Активность сияний: {:.1}/10", data.aurora_activity);
            println!("   Солнечная радиация: {:.1}", data.solar_radiation);
            println!("   Время: {}", data.timestamp.format("%H:%M UTC"));
        }
        Err(e) => {
            println!("   Error - {}", e);
        }
    }

    println!("{}", "🌌 Aurora Forecast:".bold().purple());
    
    // Прогноз северных сияний
    match predict_aurora().await {
        Ok(forecast) => {
            println!("   Вероятность: {:.1}%", forecast.visibility_probability * 100.0);
            println!("   Интенсивность: {}", forecast.intensity_level);
            println!("   Условия: {}", forecast.conditions);
            
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
                
                println!("   Лучшие часы: {}", intervals.join(", "));
            }
        }
        Err(e) => {
            println!("   Error - {}", e);
        }
    }

    Ok(())
}

async fn fetch_solar_wind_data() -> Result<SolarWindData> {
    let url = "https://services.swpc.noaa.gov/json/ace/swepam/ace_swepam_1h.json";
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}: {}", response.status(), response.text().await?));
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
    let start_idx = if all_records.len() > 50 { all_records.len() - 50 } else { 0 };
    let records = &all_records[start_idx..];
    
    // Берем последнюю запись с валидными данными
    let latest_record = records.iter()
        .filter(|r| r.dsflag == 0 && r.dens.is_some() && r.speed.is_some() && r.temperature.is_some())
        .next()
        .ok_or_else(|| anyhow::anyhow!("No valid solar wind data found"))?;
    
    let timestamp = match chrono::NaiveDateTime::parse_from_str(&latest_record.time_tag, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt.and_utc(),
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to parse timestamp '{}': {}", latest_record.time_tag, e));
        }
    };
    
    Ok(SolarWindData {
        speed: latest_record.speed.unwrap(),
        density: latest_record.dens.unwrap(),
        temperature: latest_record.temperature.unwrap(),
        magnetic_field: 0.0, // Нет данных о магнитном поле в SWEPAM
        timestamp,
    })
}

async fn fetch_geomagnetic_data() -> Result<GeomagneticData> {
    let url = "https://services.swpc.noaa.gov/json/planetary_k_index_1m.json";
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}: {}", response.status(), response.text().await?));
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
    let start_idx = if all_records.len() > 50 { all_records.len() - 50 } else { 0 };
    let records = &all_records[start_idx..];
    
    // Берем последнюю запись
    let latest_record = &records[records.len() - 1];
    
    let timestamp = match chrono::NaiveDateTime::parse_from_str(&latest_record.time_tag, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt.and_utc(),
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to parse timestamp '{}': {}", latest_record.time_tag, e));
        }
    };
    
    // Рассчитываем активность северных сияний на основе Kp индекса
    let aurora_activity = if latest_record.kp_index >= 5.0 {
        8.0 + (latest_record.kp_index - 5.0) * 0.4
    } else if latest_record.kp_index >= 3.0 {
        4.0 + (latest_record.kp_index - 3.0) * 2.0
    } else {
        latest_record.kp_index * 1.33
    }.min(10.0);
    
    Ok(GeomagneticData {
        kp_index: latest_record.kp_index,
        aurora_activity,
        solar_radiation: 0.0, // Нет данных о солнечной радиации
        timestamp,
    })
}

async fn predict_aurora() -> Result<AuroraForecast> {
    let solar_wind = fetch_solar_wind_data().await?;
    let geomagnetic = fetch_geomagnetic_data().await?;
    
    // Рассчитываем вероятность видимости северных сияний
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
    }.to_string();
    
    // Определяем условия
    let conditions = if probability > 0.6 {
        "Отличные условия для наблюдения северных сияний"
    } else if probability > 0.4 {
        "Хорошие условия, возможны сияния"
    } else if probability > 0.2 {
        "Умеренные условия, сияния маловероятны"
    } else {
        "Плохие условия для наблюдения сияний"
    }.to_string();
    
    // Определяем лучшие часы для наблюдения (ночные часы)
    let best_hours = vec![22, 23, 0, 1, 2, 3, 4, 5];
    
    Ok(AuroraForecast {
        visibility_probability: probability,
        intensity_level,
        best_viewing_hours: best_hours,
        conditions,
    })
}
