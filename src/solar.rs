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

pub struct SolarService;

impl SolarService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_solar_wind_data(&self) -> Result<SolarWindData> {
        // В реальном приложении здесь был бы вызов API NOAA или другого сервиса
        // Для демонстрации создаем моковые данные
        Ok(SolarWindData {
            speed: 400.0 + (Utc::now().timestamp() % 200) as f64,
            density: 5.0 + (Utc::now().timestamp() % 10) as f64,
            temperature: 100000.0 + (Utc::now().timestamp() % 50000) as f64,
            magnetic_field: 5.0 + (Utc::now().timestamp() % 10) as f64,
            timestamp: Utc::now(),
        })
    }

    pub async fn get_geomagnetic_data(&self) -> Result<GeomagneticData> {
        // Моковые данные для геомагнитной активности
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

pub fn print_aurora_forecast(forecast: &AuroraForecast) {
    println!("\n{}", "=== ПРОГНОЗ СЕВЕРНЫХ СИЯНИЙ ===".bold().purple());

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
