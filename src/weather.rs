use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Local};
use anyhow::Result;
use colored::*;

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

pub struct WeatherService {
    api_key: String,
    city: String,
}

impl WeatherService {
    pub fn new(api_key: String, city: String) -> Self {
        Self { api_key, city }
    }

    pub async fn get_weather_forecast(&self) -> Result<WeatherForecast> {
        // В реальном приложении здесь был бы вызов API OpenWeatherMap или другого сервиса
        // Для демонстрации создаем моковые данные
        let mut forecast = WeatherForecast { hourly: Vec::new() };
        
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 15.0 + (hour as f64 * 0.5) - 6.0, // Температура от 9 до 21 градуса
                humidity: 60.0 + (hour as f64 * 2.0) % 40.0,
                wind_speed: 5.0 + (hour as f64 * 0.3) % 15.0,
                cloud_cover: if hour < 6 || hour > 18 { 20.0 } else { 40.0 + (hour as f64 * 3.0) % 60.0 },
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
            hour_concerns.push(format!("Вероятность осадков: {}%", weather.precipitation_probability));
        }

        // Специальные условия для фотографии
        if hour >= 6 && hour <= 8 {
            hour_score += 2.0; // Золотой час утром
            hour_recommendations.push("Золотой час - идеальное время для съемки".to_string());
        } else if hour >= 18 && hour <= 20 {
            hour_score += 2.0; // Золотой час вечером
            hour_recommendations.push("Золотой час - идеальное время для съемки".to_string());
        }

        if hour_score >= 7.0 {
            analysis.best_hours.push(hour);
        }

        analysis.overall_score += hour_score;
    }

    analysis.overall_score /= 24.0;

    // Общие рекомендации
    if analysis.overall_score >= 7.0 {
        analysis.recommendations.push("Отличные условия для фотографии!".to_string());
    } else if analysis.overall_score >= 5.0 {
        analysis.recommendations.push("Хорошие условия для съемки".to_string());
    } else {
        analysis.recommendations.push("Условия не идеальны для фотографии".to_string());
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

pub fn print_weather_analysis(analysis: &WeatherAnalysis) {
    println!("\n{}", "=== АНАЛИЗ ПОГОДЫ ДЛЯ ФОТОГРАФИИ ===".bold().blue());
    
    println!("\n{}: {:.1}/10", "Общий балл".bold(), analysis.overall_score);
    
    if !analysis.recommendations.is_empty() {
        println!("\n{}:", "Рекомендации".bold().green());
        for rec in &analysis.recommendations {
            println!("  ✓ {}", rec);
        }
    }
    
    if !analysis.best_hours.is_empty() {
        println!("\n{}:", "Лучшие часы для съемки".bold().yellow());
        for hour in &analysis.best_hours {
            println!("  🕐 {}:00", hour);
        }
    }
    
    if !analysis.concerns.is_empty() {
        println!("\n{}:", "Потенциальные проблемы".bold().red());
        for concern in &analysis.concerns {
            println!("  ⚠ {}", concern);
        }
    }
}
