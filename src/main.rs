mod dashboard;
mod golden_hour;
mod photography_tips;
mod solar;
mod weather;

use colored::*;
use dashboard::PhotographyDashboard;
use golden_hour::{print_golden_hour_info, GoldenHourService};
use photography_tips::{print_photography_tips, PhotographyTipsService};
use solar::{print_aurora_forecast, SolarService};
use std::env;
use weather::{
    analyze_astrophotography_conditions, analyze_weather_for_photography,
    print_astrophotography_analysis, print_weather_analysis, WeatherService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Загружаем переменные окружения из файла .env
    dotenv::dotenv().ok();

    println!("{}", "🚀 Запуск дашборда для фотографов...".bold().blue());

    // Параметры (в реальном приложении можно получать из конфигурации)
    let api_key = env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let city = env::var("CITY").unwrap_or_else(|_| "Москва".to_string());
    let latitude = env::var("LATITUDE")
        .unwrap_or_else(|_| "55.7558".to_string())
        .parse::<f64>()
        .unwrap_or(55.7558);
    let longitude = env::var("LONGITUDE")
        .unwrap_or_else(|_| "37.6176".to_string())
        .parse::<f64>()
        .unwrap_or(37.6176);

    // Создаем дашборд
    let dashboard = PhotographyDashboard::new(api_key.clone(), city.clone(), latitude, longitude);

    // Генерируем сводку
    let summary = match dashboard.generate_dashboard().await {
        Ok(summary) => summary,
        Err(e) => {
            eprintln!("{}", "❌ ОШИБКА ГЕНЕРАЦИИ ДАШБОРДА".bold().red());
            eprintln!("Причина: {}", e);
            eprintln!(
                "{}",
                "💡 РЕШЕНИЕ: Проверьте настройки и попробуйте снова".yellow()
            );
            return Err(e);
        }
    };

    // Выводим основной дашборд
    dashboard.print_dashboard(&summary);

    // Получаем детальную информацию для каждого модуля
    println!("\n{}", "📊 ДЕТАЛЬНАЯ ИНФОРМАЦИЯ".bold().cyan());

    // Погода
    let weather_service = WeatherService::new(api_key.clone(), city.clone());
    let weather_forecast = match weather_service.get_weather_forecast().await {
        Ok(forecast) => forecast,
        Err(e) => {
            eprintln!("{}", "❌ ОШИБКА ПОЛУЧЕНИЯ ДАННЫХ ПОГОДЫ".bold().red());
            eprintln!("Причина: {}", e);
            eprintln!(
                "{}",
                "💡 РЕШЕНИЕ: Проверьте API ключ или используйте demo_key".yellow()
            );
            return Err(e.into());
        }
    };
    let weather_analysis = analyze_weather_for_photography(&weather_forecast);
    print_weather_analysis(&weather_analysis, &weather_forecast);

    // Анализ для астрофотографии
    let astrophotography_analysis = analyze_astrophotography_conditions(&weather_forecast);
    print_astrophotography_analysis(&astrophotography_analysis, &weather_forecast);

    // Северные сияния
    let solar_service = SolarService::new();
    let solar_wind_data = match solar_service.get_solar_wind_data().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "{}",
                "❌ ОШИБКА ПОЛУЧЕНИЯ ДАННЫХ СОЛНЕЧНОЙ АКТИВНОСТИ"
                    .bold()
                    .red()
            );
            eprintln!("Причина: {}", e);
            eprintln!(
                "{}",
                "💡 РЕШЕНИЕ: Используются демонстрационные данные".yellow()
            );
            return Err(e.into());
        }
    };
    let geomagnetic_data = match solar_service.get_geomagnetic_data().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", "❌ ОШИБКА ПОЛУЧЕНИЯ ГЕОМАГНИТНЫХ ДАННЫХ".bold().red());
            eprintln!("Причина: {}", e);
            eprintln!(
                "{}",
                "💡 РЕШЕНИЕ: Используются демонстрационные данные".yellow()
            );
            return Err(e.into());
        }
    };
    let aurora_forecast = solar_service.predict_aurora(&solar_wind_data, &geomagnetic_data);
    print_aurora_forecast(&aurora_forecast, &solar_wind_data, &geomagnetic_data);

    // Золотой час
    let golden_hour_service = GoldenHourService::new(latitude, longitude);
    let current_time = chrono::Local::now();
    let golden_hour_info = golden_hour_service.calculate_golden_hours(current_time);
    print_golden_hour_info(&golden_hour_info);

    // Советы для фотографов
    let tips_service = PhotographyTipsService::new();
    let is_golden_hour = summary.is_golden_hour_today;
    let photography_tips = tips_service.get_tips_for_weather(
        summary.weather_score,
        is_golden_hour,
        summary.aurora_probability,
    );
    print_photography_tips(&photography_tips);

    // Общие рекомендации
    println!("\n{}", "=== ОБЩИЕ РЕКОМЕНДАЦИИ ===".bold().white());
    let general_recommendations = tips_service.get_general_recommendations();
    for (i, rec) in general_recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, rec);
    }

    // Итог
    println!("Итог: {}", summary.overall_recommendation);
    Ok(())
}
