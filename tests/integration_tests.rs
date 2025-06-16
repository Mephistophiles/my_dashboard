use my_dashboard::{
    dashboard::{DashboardSummary, PhotographyDashboard},
    golden_hour::GoldenHourService,
    photography_tips::PhotographyTipsService,
    weather::WeatherService,
};

// NOTE: Этот тест может обращаться к сети, если не выставлен DEMO режим или demo_key
#[tokio::test]
async fn test_weather_service_forecast() {
    if std::env::var("GITHUB_ACTIONS").unwrap_or_default() == "true" {
        eprintln!("Skipped on CI");
        return;
    }
    let weather_service = WeatherService::new("demo_key".to_string(), "Moscow".to_string());
    let forecast = weather_service.get_weather_forecast().await;
    assert!(
        forecast.is_ok(),
        "Weather forecast should be fetched successfully"
    );
    let forecast = forecast.unwrap();
    assert!(
        !forecast.hourly.is_empty(),
        "Forecast should contain hourly data"
    );
}

#[test]
fn test_golden_hour_service() {
    if std::env::var("GITHUB_ACTIONS").unwrap_or_default() == "true" {
        eprintln!("Skipped on CI");
        return;
    }
    let latitude = 55.7558;
    let longitude = 37.6176;
    let golden_hour_service = GoldenHourService::new(latitude, longitude);
    let now = chrono::Local::now();
    let info = golden_hour_service.calculate_golden_hours(now);
    // Проверяем, что время восхода и заката не совпадает
    assert!(
        info.sunrise < info.sunset,
        "Sunrise should be before sunset"
    );
}

#[test]
fn test_photography_tips_service() {
    if std::env::var("GITHUB_ACTIONS").unwrap_or_default() == "true" {
        eprintln!("Skipped on CI");
        return;
    }
    let tips_service = PhotographyTipsService::new();
    let tips = tips_service.get_tips_for_weather(7.0, true, 0.8);
    // Проверяем, что советы не пустые
    assert!(!tips.equipment_recommendations.is_empty());
    assert!(
        !tips.shooting_tips.is_empty()
            || !tips.location_suggestions.is_empty()
            || !tips.technical_settings.is_empty()
    );
}

// NOTE: Этот тест может обращаться к сети, если не выставлен DEMO режим или demo_key
#[tokio::test]
async fn test_photography_dashboard_generate() {
    if std::env::var("GITHUB_ACTIONS").unwrap_or_default() == "true" {
        eprintln!("Skipped on CI");
        return;
    }
    let api_key = "demo_key".to_string();
    let city = "Moscow".to_string();
    let latitude = 55.7558;
    let longitude = 37.6176;
    let dashboard = PhotographyDashboard::new(api_key, city, latitude, longitude);
    let summary = dashboard.generate_dashboard().await;
    assert!(
        summary.is_ok(),
        "Dashboard summary should be generated successfully"
    );
    let summary: DashboardSummary = summary.unwrap();
    assert!(!summary.overall_recommendation.is_empty());
    assert!(summary.weather_score >= 0.0 && summary.weather_score <= 10.0);
    assert!(summary.aurora_probability >= 0.0 && summary.aurora_probability <= 1.0);
}
