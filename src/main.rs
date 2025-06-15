mod dashboard;
mod golden_hour;
mod photography_tips;
mod solar;
mod weather;

use colored::*;
use dashboard::PhotographyDashboard;
use golden_hour::{print_golden_hour_info, GoldenHourService};
use log::{debug, error, info, warn};
use photography_tips::{print_photography_tips, PhotographyTipsService};
use solar::{predict_aurora, print_solar_data};
use std::env;
use weather::{print_astrophotography_analysis, print_weather_analysis, WeatherService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация логирования
    env_logger::init();

    // Загружаем переменные окружения из файла .env
    dotenv::dotenv().ok();

    info!("🚀 Запуск дашборда для фотографов...");
    debug!("Отладочный режим включен");

    // Параметры (в реальном приложении можно получать из конфигурации)
    let api_key = env::var("OPENWEATHER_API_KEY").unwrap_or_else(|_| {
        warn!("OPENWEATHER_API_KEY не найден, используем demo_key");
        "demo_key".to_string()
    });

    let city = env::var("CITY").unwrap_or_else(|_| {
        info!("CITY не найден, используем Москва");
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

    debug!(
        "Параметры: город={}, широта={}, долгота={}",
        city, latitude, longitude
    );

    // Создаем дашборд
    let dashboard = PhotographyDashboard::new(api_key.clone(), city.clone(), latitude, longitude);

    // Генерируем сводку
    match dashboard.generate_dashboard().await {
        Ok(summary) => {
            dashboard.print_dashboard(&summary);
        }
        Err(e) => {
            error!("Ошибка генерации дашборда: {}", e);
            return Err(e);
        }
    }

    println!("\n{}", "📊 ДЕТАЛЬНАЯ ИНФОРМАЦИЯ".bold().cyan());

    // Переменные для хранения данных о погоде и золотом часе
    let mut weather_score = 0.0;
    let mut aurora_probability = 0.0;

    // Погода
    let weather_service = WeatherService::new(api_key, city);
    match weather_service.get_weather_forecast().await {
        Ok(forecast) => {
            debug!("Получен прогноз погоды: {} записей", forecast.hourly.len());
            print_weather_analysis(&forecast);

            // Получаем оценку погоды для советов
            let analysis = weather::analyze_weather_for_photography(&forecast);
            weather_score = analysis.overall_score;

            // Астрофотография
            print_astrophotography_analysis(&forecast);
        }
        Err(e) => {
            error!("Ошибка получения погоды: {}", e);
            println!("❌ Ошибка получения данных погоды: {}", e);
        }
    }

    // Солнечные данные и получаем вероятность сияний
    match print_solar_data().await {
        Ok(_) => debug!("Солнечные данные успешно получены"),
        Err(e) => {
            error!("Ошибка получения солнечных данных: {}", e);
            println!("❌ Ошибка получения солнечных данных: {}", e);
        }
    }

    // Получаем реальную вероятность северных сияний
    match predict_aurora().await {
        Ok(forecast) => {
            aurora_probability = forecast.visibility_probability;
            debug!(
                "Получена вероятность северных сияний: {:.0}%",
                aurora_probability * 100.0
            );
        }
        Err(e) => {
            warn!("Не удалось получить вероятность северных сияний: {}", e);
            // Оставляем значение по умолчанию 0.0
        }
    }

    // Золотой час
    let golden_hour_service = GoldenHourService::new(latitude, longitude);
    print_golden_hour_info(&golden_hour_service);

    // Проверяем, сейчас ли золотой час
    let is_golden_hour = golden_hour_service.is_golden_hour();

    println!("\n{}", "=== СОВЕТЫ ДЛЯ ФОТОГРАФОВ ===".bold().green());

    // Советы по фотографии с учетом реальных данных
    let tips_service = PhotographyTipsService::new();
    let personalized_tips =
        tips_service.get_tips_for_weather(weather_score, is_golden_hour, aurora_probability);

    // Выводим персонализированные советы
    if !personalized_tips.equipment_recommendations.is_empty() {
        println!("\n📷 РЕКОМЕНДАЦИИ ПО ОБОРУДОВАНИЮ:");
        print_photography_tips(&personalized_tips.equipment_recommendations);
    }

    if !personalized_tips.shooting_tips.is_empty() {
        println!("\n🎯 СОВЕТЫ ПО СЪЕМКЕ:");
        print_photography_tips(&personalized_tips.shooting_tips);
    }

    if !personalized_tips.location_suggestions.is_empty() {
        println!("\n📍 РЕКОМЕНДАЦИИ ПО ЛОКАЦИЯМ:");
        print_photography_tips(&personalized_tips.location_suggestions);
    }

    if !personalized_tips.technical_settings.is_empty() {
        println!("\n⚙️ ТЕХНИЧЕСКИЕ НАСТРОЙКИ:");
        print_photography_tips(&personalized_tips.technical_settings);
    }

    // Общие рекомендации
    println!("\n{}", "=== ОБЩИЕ РЕКОМЕНДАЦИИ ===".bold().blue());
    let general_tips = tips_service.get_general_recommendations();
    print_photography_tips(&general_tips);

    info!("Дашборд завершен успешно");
    Ok(())
}
