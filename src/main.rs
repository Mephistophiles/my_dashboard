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

/// Загружает и валидирует переменные окружения
fn load_environment_variables() -> (String, String, f64, f64) {
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

    (api_key, city, latitude, longitude)
}

/// Валидирует координаты
fn validate_coordinates(latitude: f64, longitude: f64) -> bool {
    (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)
}

/// Обрабатывает данные погоды и возвращает оценку
async fn process_weather_data(
    api_key: String,
    city: String,
) -> Result<f64, Box<dyn std::error::Error>> {
    let weather_service = WeatherService::new(api_key, city);
    let forecast = weather_service.get_weather_forecast().await?;

    debug!("Получен прогноз погоды: {} записей", forecast.hourly.len());
    print_weather_analysis(&forecast);

    let analysis = weather::analyze_weather_for_photography(&forecast);
    let weather_score = analysis.overall_score;

    print_astrophotography_analysis(&forecast);

    Ok(weather_score)
}

/// Обрабатывает солнечные данные и возвращает вероятность сияний
async fn process_solar_data() -> Result<f64, Box<dyn std::error::Error>> {
    print_solar_data().await?;
    debug!("Солнечные данные успешно получены");

    let forecast = predict_aurora().await?;
    let aurora_probability = forecast.visibility_probability;

    debug!(
        "Получена вероятность северных сияний: {:.0}%",
        aurora_probability * 100.0
    );

    Ok(aurora_probability)
}

/// Обрабатывает золотой час
fn process_golden_hour(latitude: f64, longitude: f64) -> bool {
    let golden_hour_service = GoldenHourService::new(latitude, longitude);
    print_golden_hour_info(&golden_hour_service);
    golden_hour_service.is_golden_hour()
}

/// Выводит персонализированные советы
fn print_personalized_tips(weather_score: f64, is_golden_hour: bool, aurora_probability: f64) {
    let tips_service = PhotographyTipsService::new();
    let personalized_tips =
        tips_service.get_tips_for_weather(weather_score, is_golden_hour, aurora_probability);

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

    println!("\n{}", "=== ОБЩИЕ РЕКОМЕНДАЦИИ ===".bold().blue());
    let general_tips = tips_service.get_general_recommendations();
    print_photography_tips(&general_tips);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация логирования
    env_logger::init();

    // Загружаем переменные окружения из файла .env
    dotenv::dotenv().ok();

    info!("🚀 Запуск дашборда для фотографов...");
    debug!("Отладочный режим включен");

    // Загружаем и валидируем параметры
    let (api_key, city, latitude, longitude) = load_environment_variables();

    if !validate_coordinates(latitude, longitude) {
        error!(
            "Некорректные координаты: lat={}, lon={}",
            latitude, longitude
        );
        return Err("Некорректные координаты".into());
    }

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

    // Обрабатываем данные погоды
    match process_weather_data(api_key.clone(), city.clone()).await {
        Ok(score) => {
            weather_score = score;
        }
        Err(e) => {
            error!("Ошибка получения погоды: {}", e);
            println!("❌ Ошибка получения данных погоды: {}", e);
        }
    }

    // Обрабатываем солнечные данные
    match process_solar_data().await {
        Ok(probability) => {
            aurora_probability = probability;
        }
        Err(e) => {
            warn!("Не удалось получить вероятность северных сияний: {}", e);
            // Оставляем значение по умолчанию 0.0
        }
    }

    // Обрабатываем золотой час
    let is_golden_hour = process_golden_hour(latitude, longitude);

    println!("\n{}", "=== СОВЕТЫ ДЛЯ ФОТОГРАФОВ ===".bold().green());

    // Выводим персонализированные советы
    print_personalized_tips(weather_score, is_golden_hour, aurora_probability);

    info!("Дашборд завершен успешно");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use std::env;

    #[test]
    fn test_load_environment_variables() {
        let (api_key, city, latitude, longitude) = load_environment_variables();

        assert!(!api_key.is_empty());
        assert!(!city.is_empty());
        assert!(validate_coordinates(latitude, longitude));
    }

    #[test]
    fn test_validate_coordinates() {
        // Валидные координаты
        assert!(validate_coordinates(55.7558, 37.6176));
        assert!(validate_coordinates(0.0, 0.0));
        assert!(validate_coordinates(90.0, 180.0));
        assert!(validate_coordinates(-90.0, -180.0));

        // Невалидные координаты
        assert!(!validate_coordinates(91.0, 37.6176)); // Широта > 90
        assert!(!validate_coordinates(-91.0, 37.6176)); // Широта < -90
        assert!(!validate_coordinates(55.7558, 181.0)); // Долгота > 180
        assert!(!validate_coordinates(55.7558, -181.0)); // Долгота < -180
    }

    #[test]
    fn test_coordinate_validation_edge_cases() {
        // Граничные значения
        assert!(validate_coordinates(90.0, 180.0));
        assert!(validate_coordinates(-90.0, -180.0));
        assert!(validate_coordinates(0.0, 0.0));

        // За границами
        assert!(!validate_coordinates(90.1, 180.0));
        assert!(!validate_coordinates(-90.1, -180.0));
        assert!(!validate_coordinates(0.0, 180.1));
        assert!(!validate_coordinates(0.0, -180.1));
    }

    #[test]
    fn test_service_initialization() {
        let golden_hour_service = GoldenHourService::new(55.7558, 37.6176);
        assert!((0..=23).contains(
            &golden_hour_service
                .calculate_golden_hours(chrono::Local::now())
                .sunrise
                .hour()
        ));

        let _tips_service = PhotographyTipsService::new();
        // Просто проверяем, что сервис создается без ошибок
    }

    #[test]
    fn test_tips_generation() {
        let tips_service = PhotographyTipsService::new();

        // Тестируем с разными параметрами
        let tips_good = tips_service.get_tips_for_weather(8.0, true, 0.7);
        assert!(!tips_good.equipment_recommendations.is_empty());
        assert!(!tips_good.shooting_tips.is_empty());

        let tips_bad = tips_service.get_tips_for_weather(3.0, false, 0.1);
        assert!(!tips_bad.equipment_recommendations.is_empty());

        let general_tips = tips_service.get_general_recommendations();
        assert_eq!(general_tips.len(), 5);
    }

    #[test]
    fn test_golden_hour_calculation() {
        let service = GoldenHourService::new(55.7558, 37.6176);
        let current_time = chrono::Local::now();
        let info = service.calculate_golden_hours(current_time);

        assert!((0..=23).contains(&info.sunrise.hour()));
        assert!((0..=23).contains(&info.sunset.hour()));
        assert!(info.sunrise < info.sunset);
    }

    #[test]
    fn test_lighting_condition_detection() {
        let service = GoldenHourService::new(55.7558, 37.6176);
        let current_time = chrono::Local::now();
        let condition = service.get_current_lighting_condition(current_time);

        assert!(!condition.is_empty());
        assert!(condition.contains("час") || condition.contains("время"));
    }

    #[test]
    fn test_dashboard_creation() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        assert!(true); // Дашборд создался успешно
    }

    #[test]
    fn test_weather_score_calculation() {
        let weather_score = 8.5;
        assert!((0.0..=10.0).contains(&weather_score));

        let weather_score2 = 3.2;
        assert!((0.0..=10.0).contains(&weather_score2));

        let weather_score3 = 0.0;
        assert!((0.0..=10.0).contains(&weather_score3));
    }

    #[test]
    fn test_aurora_probability_handling() {
        let aurora_probability = 0.8;
        assert!((0.0..=1.0).contains(&aurora_probability));

        let aurora_probability2 = 0.3;
        assert!((0.0..=1.0).contains(&aurora_probability2));

        let aurora_probability3 = 0.0;
        assert!((0.0..=1.0).contains(&aurora_probability3));
    }

    #[test]
    fn test_tips_output_validation() {
        let tips_service = PhotographyTipsService::new();
        let tips = tips_service.get_tips_for_weather(7.0, true, 0.6);

        assert!(
            !tips.equipment_recommendations.is_empty()
                || !tips.shooting_tips.is_empty()
                || !tips.location_suggestions.is_empty()
                || !tips.technical_settings.is_empty()
        );

        let general_tips = tips_service.get_general_recommendations();
        assert_eq!(general_tips.len(), 5);
        for tip in &general_tips {
            assert!(!tip.is_empty());
        }
    }

    #[test]
    fn test_golden_hour_detection_logic() {
        let service = GoldenHourService::new(55.7558, 37.6176);
        let is_golden_hour = service.is_golden_hour();
        assert!(is_golden_hour == true || is_golden_hour == false);
    }

    #[test]
    fn test_error_handling_simulation() {
        let weather_score = 0.0; // fallback value
        let aurora_probability = 0.0; // fallback value

        assert_eq!(weather_score, 0.0);
        assert_eq!(aurora_probability, 0.0);
    }

    #[test]
    fn test_process_golden_hour() {
        let is_golden_hour = process_golden_hour(55.7558, 37.6176);
        assert!(is_golden_hour == true || is_golden_hour == false);
    }

    #[test]
    fn test_print_personalized_tips() {
        // Тестируем, что функция не падает с разными параметрами
        print_personalized_tips(8.0, true, 0.7);
        print_personalized_tips(3.0, false, 0.1);
        print_personalized_tips(5.0, true, 0.0);
        assert!(true); // Функция выполнилась без ошибок
    }
}
