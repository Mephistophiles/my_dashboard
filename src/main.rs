use colored::*;
use log::{debug, error, info};
use my_dashboard::{
    generate_dashboard_output, load_environment_variables, validate_coordinates, DashboardOutput,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Инициализация логирования
    env_logger::init();

    // Загружаем переменные окружения из файла .env
    dotenv::dotenv().ok();

    info!("🚀 Запуск дашборда для фотографов...");

    // Загружаем и валидируем параметры
    let (api_key, city, latitude, longitude) = load_environment_variables();

    if !validate_coordinates(latitude, longitude) {
        error!(
            "Некорректные координаты: lat={}, lon={}",
            latitude, longitude
        );
        return Ok(());
    }

    debug!(
        "Параметры: город={}, широта={}, долгота={}",
        city, latitude, longitude
    );

    // Генерируем весь дашборд
    let dashboard_output = match generate_dashboard_output(api_key, city, latitude, longitude).await
    {
        Ok(output) => output,
        Err(e) => {
            error!("Ошибка генерации дашборда: {}", e);
            return Ok(());
        }
    };

    // Выводим результаты
    print_dashboard_output(&dashboard_output);

    info!("Дашборд завершен успешно");
    Ok(())
}

fn print_dashboard_output(output: &DashboardOutput) {
    // Выводим основную сводку дашборда
    print_dashboard_summary(&output.summary);

    println!("\n{}", "📊 ДЕТАЛЬНАЯ ИНФОРМАЦИЯ".bold().cyan());

    // Выводим данные погоды
    println!("{}", output.weather_output.current_weather);
    print!("{}  ", output.weather_output.temperature_range);
    print!("{}  ", output.weather_output.best_hours);
    println!("| ⭐ Оценка: {:.1}/10", output.weather_output.overall_score);

    if !output.weather_output.recommendation.is_empty() {
        print!("{}", output.weather_output.recommendation);
    }

    if !output.weather_output.concerns.is_empty() {
        print!(" | {}", output.weather_output.concerns);
    }
    println!();

    // Выводим анализ астрофотографии
    print!(
        "🌌 Астрофото: {} | ☁️{:.0}% | ",
        if output.astrophotography_output.is_suitable {
            "✅"
        } else {
            "❌"
        },
        output.astrophotography_output.avg_cloud_cover
    );

    if !output.astrophotography_output.best_hours.is_empty() {
        print!("{} ", output.astrophotography_output.best_hours);
    }

    if !output.astrophotography_output.recommendation.is_empty() {
        print!("| {}", output.astrophotography_output.recommendation);
    }
    println!();

    // Выводим солнечные данные
    println!("{}", output.solar_output.solar_wind);
    println!("{}", output.solar_output.geomagnetic);
    println!("{}", output.solar_output.aurora_forecast);
    if !output.solar_output.best_viewing_hours.is_empty() {
        println!("   {}", output.solar_output.best_viewing_hours);
    }

    // Выводим информацию о золотом часе
    println!("{}", output.golden_hour_output.sunrise_sunset);
    println!("{}", output.golden_hour_output.golden_hours);
    println!("{}", output.golden_hour_output.blue_hours);
    println!(
        "💡 Текущие условия освещения: {}",
        output.golden_hour_output.current_condition
    );

    println!("\n{}", "=== СОВЕТЫ ДЛЯ ФОТОГРАФОВ ===".bold().green());

    // Выводим персонализированные советы
    print_personalized_tips(&output.tips_output);
}

fn print_dashboard_summary(summary: &my_dashboard::dashboard::DashboardSummary) {
    println!("\n{}", "=== ФОТОГРАФИЧЕСКИЙ ДАШБОРД ===".bold().white());
    println!("{}", "📊 ОБЩАЯ ОЦЕНКА".bold().cyan());
    println!("   Погода: {:.1}/10", summary.weather_score);
    println!(
        "   Вероятность северных сияний: {:.0}%",
        summary.aurora_probability * 100.0
    );
    println!(
        "   Золотой час: {}",
        if summary.is_golden_hour_today {
            "Да"
        } else {
            "Нет"
        }
    );

    if !summary.best_shooting_hours.is_empty() {
        // Сжимаем часы до интервалов
        let mut intervals = Vec::new();
        let mut start = summary.best_shooting_hours[0];
        let mut end = start;

        for &hour in &summary.best_shooting_hours[1..] {
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
        // Добавляем последний интервал
        if start == end {
            intervals.push(format!("{:02}:00", start));
        } else {
            intervals.push(format!("{:02}:00-{:02}:00", start, end));
        }

        println!("   Лучшие часы: {}", intervals.join(", "));
    }

    if !summary.key_highlights.is_empty() {
        println!("{}", "✨ КЛЮЧЕВЫЕ МОМЕНТЫ".bold().green());
        for highlight in &summary.key_highlights {
            println!("   • {}", highlight);
        }
    }

    if !summary.warnings.is_empty() {
        println!("{}", "⚠️ ПРЕДУПРЕЖДЕНИЯ".bold().yellow());
        for warning in &summary.warnings {
            println!("   • {}", warning);
        }
    }

    println!("{}", "🎯 РЕКОМЕНДАЦИЯ".bold().blue());
    println!("   {}", summary.overall_recommendation);
}

fn print_personalized_tips(tips_output: &my_dashboard::PhotographyTipsOutput) {
    if !tips_output.equipment_recommendations.is_empty() {
        println!("\n📷 РЕКОМЕНДАЦИИ ПО ОБОРУДОВАНИЮ:");
        for (i, tip) in tips_output.equipment_recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, tip);
        }
    }

    if !tips_output.shooting_tips.is_empty() {
        println!("\n🎯 СОВЕТЫ ПО СЪЕМКЕ:");
        for (i, tip) in tips_output.shooting_tips.iter().enumerate() {
            println!("{}. {}", i + 1, tip);
        }
    }

    if !tips_output.location_suggestions.is_empty() {
        println!("\n📍 РЕКОМЕНДАЦИИ ПО ЛОКАЦИЯМ:");
        for (i, tip) in tips_output.location_suggestions.iter().enumerate() {
            println!("{}. {}", i + 1, tip);
        }
    }

    if !tips_output.technical_settings.is_empty() {
        println!("\n⚙️ ТЕХНИЧЕСКИЕ НАСТРОЙКИ:");
        for (i, tip) in tips_output.technical_settings.iter().enumerate() {
            println!("{}. {}", i + 1, tip);
        }
    }

    println!("\n{}", "=== ОБЩИЕ РЕКОМЕНДАЦИИ ===".bold().blue());
    for (i, tip) in tips_output.general_recommendations.iter().enumerate() {
        println!("{}. {}", i + 1, tip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(validate_coordinates(55.7558, 37.6176)); // Москва
        assert!(validate_coordinates(0.0, 0.0)); // Экватор, нулевой меридиан
        assert!(validate_coordinates(90.0, 180.0)); // Северный полюс, максимальная долгота
        assert!(validate_coordinates(-90.0, -180.0)); // Южный полюс, минимальная долгота

        // Невалидные координаты
        assert!(!validate_coordinates(91.0, 37.6176)); // Широта > 90
        assert!(!validate_coordinates(-91.0, 37.6176)); // Широта < -90
        assert!(!validate_coordinates(55.7558, 181.0)); // Долгота > 180
        assert!(!validate_coordinates(55.7558, -181.0)); // Долгота < -180
    }

    #[test]
    fn test_validate_coordinates_comprehensive() {
        // Тестируем граничные случаи
        assert!(validate_coordinates(90.0, 180.0));
        assert!(validate_coordinates(-90.0, -180.0));
        assert!(validate_coordinates(0.0, 0.0));
        assert!(validate_coordinates(45.0, 90.0));
        assert!(validate_coordinates(-45.0, -90.0));

        // Тестируем невалидные случаи
        assert!(!validate_coordinates(90.1, 180.0));
        assert!(!validate_coordinates(-90.1, -180.0));
        assert!(!validate_coordinates(0.0, 180.1));
        assert!(!validate_coordinates(0.0, -180.1));
    }
}
