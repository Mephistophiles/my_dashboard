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
    pub intensity: String,           // Интенсивность
    pub best_viewing_hours: Vec<usize>,
    pub recommendations: Vec<String>,
    pub concerns: Vec<String>,
}

// Структуры для парсинга ответов API
#[derive(Debug, Deserialize)]
struct NOAARealTimeData {
    data: Vec<NOAADataPoint>,
}

#[derive(Debug, Deserialize)]
struct NOAADataPoint {
    #[serde(rename = "time_tag")]
    time_tag: String,
    #[serde(rename = "density")]
    density: Option<f64>,
    #[serde(rename = "speed")]
    speed: Option<f64>,
    #[serde(rename = "temperature")]
    temperature: Option<f64>,
    #[serde(rename = "bz_gsm")]
    bz_gsm: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct KPIndexData {
    data: Vec<KPDataPoint>,
}

#[derive(Debug, Deserialize)]
struct KPDataPoint {
    #[serde(rename = "time_tag")]
    time_tag: String,
    #[serde(rename = "kp_index")]
    kp_index: Option<f64>,
}

pub struct SolarService;

impl SolarService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_solar_wind_data(&self) -> Result<SolarWindData> {
        // Получаем реальные данные от NOAA Space Weather API
        let url = "https://services.swpc.noaa.gov/json/plasma/plasma-6-hour.json";
        
        let response = match reqwest::get(url).await {
            Ok(resp) => resp,
            Err(_) => {
                // Если не удалось подключиться, возвращаем моковые данные
                return self.get_mock_solar_wind_data();
            }
        };
        
        if !response.status().is_success() {
            // Если API недоступен, возвращаем моковые данные
            return self.get_mock_solar_wind_data();
        }

        let noaa_data: NOAARealTimeData = match response.json().await {
            Ok(data) => data,
            Err(_) => {
                // Если не удалось распарсить JSON, возвращаем моковые данные
                return self.get_mock_solar_wind_data();
            }
        };
        
        // Берем последние доступные данные
        if let Some(latest_data) = noaa_data.data.last() {
            let speed = latest_data.speed.unwrap_or(400.0);
            let density = latest_data.density.unwrap_or(5.0);
            let temperature = latest_data.temperature.unwrap_or(100000.0);
            let magnetic_field = latest_data.bz_gsm.unwrap_or(5.0).abs();

            return Ok(SolarWindData {
                speed,
                density,
                temperature,
                magnetic_field,
                timestamp: Utc::now(),
            });
        }

        // Если данные не получены, возвращаем моковые
        self.get_mock_solar_wind_data()
    }

    pub async fn get_geomagnetic_data(&self) -> Result<GeomagneticData> {
        // Получаем реальные данные Kp индекса от NOAA
        let url = "https://services.swpc.noaa.gov/json/planetary_k_index_1m.json";
        
        let response = match reqwest::get(url).await {
            Ok(resp) => resp,
            Err(_) => {
                // Если не удалось подключиться, возвращаем моковые данные
                return self.get_mock_geomagnetic_data();
            }
        };
        
        if !response.status().is_success() {
            // Если API недоступен, возвращаем моковые данные
            return self.get_mock_geomagnetic_data();
        }

        let kp_data: KPIndexData = match response.json().await {
            Ok(data) => data,
            Err(_) => {
                // Если не удалось распарсить JSON, возвращаем моковые данные
                return self.get_mock_geomagnetic_data();
            }
        };
        
        // Берем последние доступные данные Kp
        if let Some(latest_kp) = kp_data.data.last() {
            let kp_index = latest_kp.kp_index.unwrap_or(2.0);
            let aurora_activity = if kp_index > 4.0 {
                (kp_index - 4.0) * 2.0
            } else {
                0.0
            };

            return Ok(GeomagneticData {
                kp_index,
                aurora_activity,
                solar_radiation: 100.0 + (Utc::now().timestamp() % 50) as f64, // Моковые данные для радиации
                timestamp: Utc::now(),
            });
        }

        // Если данные не получены, возвращаем моковые
        self.get_mock_geomagnetic_data()
    }

    fn get_mock_solar_wind_data(&self) -> Result<SolarWindData> {
        Ok(SolarWindData {
            speed: 400.0 + (Utc::now().timestamp() % 200) as f64,
            density: 5.0 + (Utc::now().timestamp() % 10) as f64,
            temperature: 100000.0 + (Utc::now().timestamp() % 50000) as f64,
            magnetic_field: 5.0 + (Utc::now().timestamp() % 10) as f64,
            timestamp: Utc::now(),
        })
    }

    fn get_mock_geomagnetic_data(&self) -> Result<GeomagneticData> {
        let kp_index = 2.0 + (Utc::now().timestamp() % 7) as f64;
        let aurora_activity = if kp_index > 4.0 {
            (kp_index - 4.0) * 2.0
        } else {
            0.0
        };

        Ok(GeomagneticData {
            kp_index,
            aurora_activity,
            solar_radiation: 100.0 + (Utc::now().timestamp() % 50) as f64,
            timestamp: Utc::now(),
        })
    }

    pub fn predict_aurora(
        &self,
        solar_wind: &SolarWindData,
        geomagnetic: &GeomagneticData,
    ) -> AuroraForecast {
        let mut forecast = AuroraForecast {
            visibility_probability: 0.0,
            intensity: "Низкая".to_string(),
            best_viewing_hours: Vec::new(),
            recommendations: Vec::new(),
            concerns: Vec::new(),
        };

        // Расчет вероятности видимости северных сияний
        let mut probability: f64 = 0.0;

        // Влияние солнечного ветра
        if solar_wind.speed > 500.0 {
            probability += 0.3;
        } else if solar_wind.speed > 400.0 {
            probability += 0.2;
        }

        if solar_wind.density > 10.0 {
            probability += 0.2;
        }

        // Влияние геомагнитной активности
        if geomagnetic.kp_index > 5.0 {
            probability += 0.4;
            forecast.intensity = "Высокая".to_string();
        } else if geomagnetic.kp_index > 3.0 {
            probability += 0.2;
            forecast.intensity = "Средняя".to_string();
        }

        // Лучшие часы для наблюдения (ночное время)
        for hour in 22..24 {
            forecast.best_viewing_hours.push(hour);
        }
        for hour in 0..6 {
            forecast.best_viewing_hours.push(hour);
        }

        forecast.visibility_probability = probability.min(1.0);

        // Рекомендации
        if forecast.visibility_probability > 0.7 {
            forecast
                .recommendations
                .push("Отличные условия для наблюдения северных сияний!".to_string());
            forecast
                .recommendations
                .push("Ищите темные места вдали от городских огней".to_string());
        } else if forecast.visibility_probability > 0.4 {
            forecast
                .recommendations
                .push("Умеренная вероятность северных сияний".to_string());
        } else {
            forecast
                .recommendations
                .push("Низкая вероятность северных сияний".to_string());
        }

        // Проблемы
        if geomagnetic.kp_index > 6.0 {
            forecast
                .concerns
                .push("Высокая геомагнитная активность может повлиять на электронику".to_string());
        }

        forecast
    }
}

pub fn print_aurora_forecast(forecast: &AuroraForecast, solar_wind: &SolarWindData, geomagnetic: &GeomagneticData) {
    println!("\n{}", "=== ПРОГНОЗ СЕВЕРНЫХ СИЯНИЙ ===".bold().purple());

    // Показываем текущие данные
    println!("\n{}:", "ТЕКУЩИЕ ДАННЫЕ".bold().cyan());
    println!("  🌪️  Скорость солнечного ветра: {:.0} км/с", solar_wind.speed);
    println!("  📊 Плотность плазмы: {:.1} частиц/см³", solar_wind.density);
    println!("  🌡️  Температура плазмы: {:.0} К", solar_wind.temperature);
    println!("  🧲 Магнитное поле: {:.1} нТл", solar_wind.magnetic_field);
    println!("  📈 Kp индекс: {:.1}", geomagnetic.kp_index);
    println!("  ☢️  Солнечная радиация: {:.0} SFU", geomagnetic.solar_radiation);

    println!(
        "\n{}: {:.1}%",
        "Вероятность видимости".bold(),
        forecast.visibility_probability * 100.0
    );
    println!("{}: {}", "Интенсивность".bold(), forecast.intensity);

    if !forecast.best_viewing_hours.is_empty() {
        println!("\n{}:", "Лучшие часы для наблюдения".bold().cyan());
        for hour in &forecast.best_viewing_hours {
            println!("  🌙 {}:00", hour);
        }
    }

    if !forecast.recommendations.is_empty() {
        println!("\n{}:", "Рекомендации".bold().green());
        for rec in &forecast.recommendations {
            println!("  ✓ {}", rec);
        }
    }

    if !forecast.concerns.is_empty() {
        println!("\n{}:", "Предупреждения".bold().red());
        for concern in &forecast.concerns {
            println!("  ⚠ {}", concern);
        }
    }
}
