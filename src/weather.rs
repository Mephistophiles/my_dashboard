use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::*;
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
}

impl WeatherService {
    pub fn new(api_key: String, city: String) -> Self {
        Self { api_key, city }
    }

    pub async fn get_weather_forecast(&self) -> Result<WeatherForecast> {
        // Если используется demo_key, возвращаем моковые данные
        if self.api_key == "demo_key" {
            return self.get_mock_forecast();
        }

        // Получаем координаты города
        let coords = self.get_city_coordinates().await?;

        // Используем бесплатный Current Weather API вместо OneCall
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={}",
            coords.lat, coords.lon, self.api_key
        );

        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_message = match status.as_u16() {
                401 => {
                    format!(
                        "Неверный API ключ. Получите бесплатный ключ на https://openweathermap.org/api"
                    )
                }
                429 => {
                    "Превышен лимит запросов. Попробуйте позже.".to_string()
                }
                404 => {
                    format!("Город '{}' не найден", self.city)
                }
                _ => {
                    format!("HTTP ошибка {} при получении данных погоды", status)
                }
            };
            return Err(anyhow::anyhow!(error_message));
        }

        let weather_response: CurrentWeatherResponse = response.json().await?;

        // Создаем прогноз на основе текущих данных
        let mut forecast = WeatherForecast { hourly: Vec::new() };
        
        // Генерируем прогноз на 24 часа на основе текущих данных
        let current_time = chrono::Utc::now();
        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: weather_response.main.temp,
                humidity: weather_response.main.humidity,
                wind_speed: weather_response.wind.speed,
                cloud_cover: weather_response.clouds.all,
                visibility: weather_response.visibility / 1000.0, // конвертируем в км
                precipitation_probability: 0.0, // нет данных о вероятности осадков в current weather
                description: weather_response
                    .weather
                    .first()
                    .map(|w| w.description.clone())
                    .unwrap_or_else(|| "Неизвестно".to_string()),
                timestamp: current_time + chrono::Duration::hours(hour),
            };
            forecast.hourly.push(weather_data);
        }

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
        // Моковые данные для демонстрации
        let mut forecast = WeatherForecast { hourly: Vec::new() };

        for hour in 0..24 {
            let weather_data = WeatherData {
                temperature: 15.0 + (hour as f64 * 0.5) - 6.0, // Температура от 9 до 21 градуса
                humidity: 60.0 + (hour as f64 * 2.0) % 40.0,
                wind_speed: 5.0 + (hour as f64 * 0.3) % 15.0,
                cloud_cover: if !(6..=18).contains(&hour) {
                    20.0
                } else {
                    40.0 + (hour as f64 * 3.0) % 60.0
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
        visibility_issues: Vec::new(),
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
            hour_concerns.push(format!("Облачность {}% - не подходит для астрофотографии", weather.cloud_cover));
        }

        // Проверяем видимость
        if weather.visibility < 10.0 {
            hour_suitable = false;
            hour_concerns.push(format!("Плохая видимость {} км", weather.visibility));
        }

        // Проверяем осадки
        if weather.precipitation_probability > 10.0 {
            hour_suitable = false;
            hour_concerns.push(format!("Вероятность осадков {}%", weather.precipitation_probability));
        }

        // Проверяем ветер (может влиять на качество снимков)
        if weather.wind_speed > 15.0 {
            hour_concerns.push(format!("Сильный ветер {} м/с может влиять на качество", weather.wind_speed));
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
        analysis.recommendations.push("Отличные условия для астрофотографии!".to_string());
        analysis.recommendations.push("Ищите темные места вдали от городских огней".to_string());
        analysis.recommendations.push("Используйте штатив для длительных экспозиций".to_string());
    } else {
        analysis.recommendations.push("Условия не подходят для астрофотографии".to_string());
        analysis.recommendations.push("Рекомендуется перенести съемку на другой день".to_string());
    }

    // Проверяем общую облачность
    let avg_cloud_cover = forecast.hourly.iter().map(|w| w.cloud_cover).sum::<f64>() / forecast.hourly.len() as f64;
    if avg_cloud_cover > 50.0 {
        analysis.concerns.push(format!("Высокая средняя облачность {}% - неблагоприятно для астрофотографии", avg_cloud_cover));
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

pub fn print_weather_analysis(analysis: &WeatherAnalysis, forecast: &WeatherForecast) {
    println!("\n{}", "=== АНАЛИЗ ПОГОДЫ ДЛЯ ФОТОГРАФИИ ===".bold().blue());

    // Показываем текущие погодные условия
    if let Some(current_weather) = forecast.hourly.first() {
        println!("\n{}:", "ТЕКУЩИЕ УСЛОВИЯ".bold().cyan());
        println!("  🌡️  Температура: {:.1}°C", current_weather.temperature);
        println!("  💧 Влажность: {:.0}%", current_weather.humidity);
        println!("  💨 Ветер: {:.1} м/с", current_weather.wind_speed);
        println!("  ☁️  Облачность: {:.0}%", current_weather.cloud_cover);
        println!("  👁️  Видимость: {:.1} км", current_weather.visibility);
        println!("  🌧️  Вероятность осадков: {:.0}%", current_weather.precipitation_probability);
        println!("  📝 Описание: {}", current_weather.description);
    }

    println!(
        "\n{}: {:.1}/10",
        "Общий балл".bold(),
        analysis.overall_score
    );

    // Показываем диапазон температур за день
    if !forecast.hourly.is_empty() {
        let min_temp = forecast.hourly.iter().map(|w| w.temperature).fold(f64::INFINITY, f64::min);
        let max_temp = forecast.hourly.iter().map(|w| w.temperature).fold(f64::NEG_INFINITY, f64::max);
        let avg_temp = forecast.hourly.iter().map(|w| w.temperature).sum::<f64>() / forecast.hourly.len() as f64;
        
        println!("\n{}:", "ТЕМПЕРАТУРНЫЙ РЕЖИМ".bold().magenta());
        println!("  📊 Минимум: {:.1}°C", min_temp);
        println!("  📊 Максимум: {:.1}°C", max_temp);
        println!("  📊 Средняя: {:.1}°C", avg_temp);
        
        // Оценка температурного режима
        if min_temp >= 10.0 && max_temp <= 25.0 {
            println!("  ✅ Комфортный температурный диапазон для съемки");
        } else if min_temp >= 5.0 && max_temp <= 30.0 {
            println!("  ⚠️  Приемлемый температурный диапазон");
        } else {
            println!("  ❌ Неудобный температурный диапазон для съемки");
        }
    }

    // Показываем информацию об осадках
    if !forecast.hourly.is_empty() {
        let max_precipitation = forecast.hourly.iter().map(|w| w.precipitation_probability).fold(0.0, f64::max);
        let hours_with_precipitation = forecast.hourly.iter().filter(|w| w.precipitation_probability > 20.0).count();
        
        println!("\n{}:", "ОСАДКИ".bold().blue());
        println!("  🌧️  Максимальная вероятность осадков: {:.0}%", max_precipitation);
        println!("  ⏰ Часов с вероятностью осадков >20%: {}", hours_with_precipitation);
        
        if max_precipitation < 20.0 {
            println!("  ✅ Благоприятные условия - низкая вероятность осадков");
        } else if max_precipitation < 50.0 {
            println!("  ⚠️  Умеренная вероятность осадков");
        } else {
            println!("  ❌ Высокая вероятность осадков - неблагоприятно для съемки");
        }
    }

    // Показываем информацию о ветре
    if !forecast.hourly.is_empty() {
        let max_wind = forecast.hourly.iter().map(|w| w.wind_speed).fold(0.0, f64::max);
        let avg_wind = forecast.hourly.iter().map(|w| w.wind_speed).sum::<f64>() / forecast.hourly.len() as f64;
        
        println!("\n{}:", "ВЕТЕР".bold().yellow());
        println!("  💨 Максимальная скорость: {:.1} м/с", max_wind);
        println!("  💨 Средняя скорость: {:.1} м/с", avg_wind);
        
        if max_wind < 10.0 {
            println!("  ✅ Благоприятные ветровые условия");
        } else if max_wind < 15.0 {
            println!("  ⚠️  Умеренный ветер - может потребоваться штатив");
        } else {
            println!("  ❌ Сильный ветер - неблагоприятно для съемки");
        }
    }

    if !analysis.recommendations.is_empty() {
        println!("\n{}:", "Рекомендации".bold().green());
        for rec in &analysis.recommendations {
            println!("  ✓ {}", rec);
        }
    }

    if !analysis.best_hours.is_empty() {
        println!("\n{}:", "Лучшие часы для съемки".bold().yellow());
        for hour in &analysis.best_hours {
            if let Some(weather) = forecast.hourly.get(*hour) {
                println!("  🕐 {}:00 - {}°C, ветер {:.1} м/с, облачность {:.0}%", 
                    hour, weather.temperature, weather.wind_speed, weather.cloud_cover);
            } else {
                println!("  🕐 {}:00", hour);
            }
        }
    }

    if !analysis.concerns.is_empty() {
        println!("\n{}:", "Потенциальные проблемы".bold().red());
        for concern in &analysis.concerns {
            println!("  ⚠ {}", concern);
        }
    }

    // Итоговая оценка
    println!("\n{}:", "ИТОГОВАЯ ОЦЕНКА".bold().white());
    if analysis.overall_score >= 8.0 {
        println!("  🎯 ОТЛИЧНО! Идеальные условия для фотографии");
    } else if analysis.overall_score >= 6.0 {
        println!("  👍 ХОРОШО! Подходящие условия для съемки");
    } else if analysis.overall_score >= 4.0 {
        println!("  ⚠️  УДОВЛЕТВОРИТЕЛЬНО! Условия приемлемы, но не идеальны");
    } else {
        println!("  ❌ НЕБЛАГОПРИЯТНО! Условия не подходят для качественной съемки");
    }
}

#[derive(Debug)]
pub struct AstrophotographyAnalysis {
    pub is_suitable: bool,
    pub cloud_cover_issues: Vec<String>,
    pub visibility_issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub best_hours: Vec<usize>,
    pub concerns: Vec<String>,
}

pub fn print_astrophotography_analysis(analysis: &AstrophotographyAnalysis, forecast: &WeatherForecast) {
    println!("\n{}", "=== АНАЛИЗ ДЛЯ АСТРОФОТОГРАФИИ ===".bold().purple());

    if analysis.is_suitable {
        println!("\n{}", "✅ УСЛОВИЯ ПОДХОДЯТ ДЛЯ АСТРОФОТОГРАФИИ".bold().green());
    } else {
        println!("\n{}", "❌ УСЛОВИЯ НЕ ПОДХОДЯТ ДЛЯ АСТРОФОТОГРАФИИ".bold().red());
    }

    // Показываем статистику облачности
    if !forecast.hourly.is_empty() {
        let avg_cloud_cover = forecast.hourly.iter().map(|w| w.cloud_cover).sum::<f64>() / forecast.hourly.len() as f64;
        let min_cloud_cover = forecast.hourly.iter().map(|w| w.cloud_cover).fold(f64::INFINITY, f64::min);
        let max_cloud_cover = forecast.hourly.iter().map(|w| w.cloud_cover).fold(f64::NEG_INFINITY, f64::max);
        
        println!("\n{}:", "ОБЛАЧНОСТЬ".bold().blue());
        println!("  ☁️  Средняя облачность: {:.0}%", avg_cloud_cover);
        println!("  ☁️  Минимальная облачность: {:.0}%", min_cloud_cover);
        println!("  ☁️  Максимальная облачность: {:.0}%", max_cloud_cover);
        
        if avg_cloud_cover > 50.0 {
            println!("  ⚠️  Высокая облачность - неблагоприятно для астрофотографии");
        } else if avg_cloud_cover > 20.0 {
            println!("  ⚠️  Умеренная облачность - могут быть проблемы");
        } else {
            println!("  ✅ Низкая облачность - отлично для астрофотографии");
        }
    }

    if !analysis.best_hours.is_empty() {
        println!("\n{}:", "Лучшие часы для астрофотографии".bold().cyan());
        for hour in &analysis.best_hours {
            if let Some(weather) = forecast.hourly.get(*hour) {
                println!("  🌙 {}:00 - облачность {:.0}%, видимость {:.1} км", 
                    hour, weather.cloud_cover, weather.visibility);
            } else {
                println!("  🌙 {}:00", hour);
            }
        }
    }

    if !analysis.recommendations.is_empty() {
        println!("\n{}:", "Рекомендации".bold().green());
        for rec in &analysis.recommendations {
            println!("  ✓ {}", rec);
        }
    }

    if !analysis.concerns.is_empty() {
        println!("\n{}:", "Проблемы".bold().red());
        for concern in &analysis.concerns {
            println!("  ⚠ {}", concern);
        }
    }

    // Специальные советы для астрофотографии
    println!("\n{}:", "СОВЕТЫ ДЛЯ АСТРОФОТОГРАФИИ".bold().yellow());
    println!("  📸 Используйте штатив для стабильности");
    println!("  📸 Установите высокое ISO (800-3200)");
    println!("  📸 Используйте широкую диафрагму (f/2.8-f/4)");
    println!("  📸 Делайте длительные экспозиции (15-30 секунд)");
    println!("  📸 Ищите места вдали от городских огней");
    println!("  📸 Проверяйте фазу Луны - полнолуние может мешать");
}
