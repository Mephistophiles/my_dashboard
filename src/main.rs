mod dashboard;
mod golden_hour;
mod photography_tips;
mod solar;
mod weather;

use colored::*;
use dashboard::PhotographyDashboard;
use golden_hour::{print_golden_hour_info, GoldenHourService};
use log::{debug, info, warn, error};
use photography_tips::{print_photography_tips, PhotographyTipsService};
use solar::print_solar_data;
use std::env;
use weather::{
    print_astrophotography_analysis, print_weather_analysis, WeatherService,
};

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

    debug!("Параметры: город={}, широта={}, долгота={}", city, latitude, longitude);

    // Создаем дашборд
    let dashboard = PhotographyDashboard::new(api_key.clone(), city.clone(), latitude, longitude);
    
    // Генерируем сводку
    match dashboard.generate_dashboard().await {
        Ok(summary) => {
            dashboard.print_dashboard(&summary);
        }
        Err(e) => {
            error!("Ошибка генерации дашборда: {}", e);
            return Err(e.into());
        }
    }

    println!("\n{}", "📊 ДЕТАЛЬНАЯ ИНФОРМАЦИЯ".bold().cyan());

    // Погода
    let weather_service = WeatherService::new(api_key, city);
    match weather_service.get_weather_forecast().await {
        Ok(forecast) => {
            debug!("Получен прогноз погоды: {} записей", forecast.hourly.len());
            print_weather_analysis(&forecast);
            
            // Астрофотография
            print_astrophotography_analysis(&forecast);
        }
        Err(e) => {
            error!("Ошибка получения погоды: {}", e);
            println!("❌ Ошибка получения данных погоды: {}", e);
        }
    }

    // Солнечные данные
    match print_solar_data().await {
        Ok(_) => debug!("Солнечные данные успешно получены"),
        Err(e) => {
            error!("Ошибка получения солнечных данных: {}", e);
            println!("❌ Ошибка получения солнечных данных: {}", e);
        }
    }

    // Золотой час
    let golden_hour_service = GoldenHourService::new(latitude, longitude);
    print_golden_hour_info(&golden_hour_service);

    println!("\n{}", "=== СОВЕТЫ ДЛЯ ФОТОГРАФОВ ===".bold().green());

    // Советы по фотографии
    let tips_service = PhotographyTipsService::new();
    let tips = tips_service.get_general_recommendations();
    print_photography_tips(&tips);

    println!("\n{}", "=== ОБЩИЕ РЕКОМЕНДАЦИИ ===".bold().blue());
    println!("  1. Всегда проверяйте прогноз погоды перед съемкой");
    println!("  2. Планируйте локации заранее");
    println!("  3. Берите запасные батареи и карты памяти");
    println!("  4. Изучите правила съемки в выбранных местах");
    println!("  5. Не забудьте о безопасности - особенно при съемке в дикой природе");

    info!("Дашборд завершен успешно");
    Ok(())
}
