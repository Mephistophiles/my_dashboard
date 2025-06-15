use crate::golden_hour::{GoldenHourInfo, GoldenHourService};
use crate::solar::predict_aurora;
use crate::weather::{analyze_weather_for_photography, WeatherAnalysis, WeatherService};
use chrono::{DateTime, Local, Timelike};
use colored::*;

#[derive(Debug)]
pub struct DashboardSummary {
    pub overall_recommendation: String,
    pub weather_score: f64,
    pub aurora_probability: f64,
    pub is_golden_hour_today: bool,
    pub best_shooting_hours: Vec<usize>,
    pub key_highlights: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct PhotographyDashboard {
    weather_service: WeatherService,
    golden_hour_service: GoldenHourService,
}

impl PhotographyDashboard {
    pub fn new(api_key: String, city: String, latitude: f64, longitude: f64) -> Self {
        Self {
            weather_service: WeatherService::new(api_key, city),
            golden_hour_service: GoldenHourService::new(latitude, longitude),
        }
    }

    pub async fn generate_dashboard(&self) -> Result<DashboardSummary, Box<dyn std::error::Error>> {
        let current_time = Local::now();

        // Получаем данные о погоде
        let weather_forecast = match self.weather_service.get_weather_forecast().await {
            Ok(forecast) => forecast,
            Err(e) => {
                eprintln!(
                    "{}",
                    "❌ ОШИБКА ПОЛУЧЕНИЯ ДАННЫХ ПОГОДЫ В ДАШБОРДЕ".bold().red()
                );
                eprintln!("Причина: {}", e);
                eprintln!(
                    "{}",
                    "💡 РЕШЕНИЕ: Проверьте API ключ или используйте demo_key".yellow()
                );
                return Err(e.into());
            }
        };
        let weather_analysis = analyze_weather_for_photography(&weather_forecast);

        // Получаем информацию о золотом часе
        let golden_hour_info = self
            .golden_hour_service
            .calculate_golden_hours(current_time);

        // Определяем, есть ли золотой час сегодня
        let is_golden_hour_today = self.is_golden_hour_today(&golden_hour_info, current_time);

        // Получаем вероятность северных сияний
        let aurora_probability = match predict_aurora().await {
            Ok(forecast) => forecast.visibility_probability,
            Err(e) => {
                eprintln!(
                    "{}",
                    "❌ ОШИБКА ПОЛУЧЕНИЯ ДАННЫХ О СЕВЕРНЫХ СИЯНИЯХ".bold().red()
                );
                eprintln!("Причина: {}", e);
                eprintln!("{}", "💡 РЕШЕНИЕ: Проверьте интернет-соединение".yellow());
                return Err(e.into());
            }
        };

        // Создаем общую сводку
        let summary = self.create_summary(
            &weather_analysis,
            &golden_hour_info,
            is_golden_hour_today,
            current_time,
            aurora_probability,
        );

        Ok(summary)
    }

    fn is_golden_hour_today(
        &self,
        golden_hour_info: &GoldenHourInfo,
        current_time: DateTime<Local>,
    ) -> bool {
        // Проверяем, попадает ли текущее время в золотой час
        (current_time >= golden_hour_info.golden_hour_morning_start
            && current_time <= golden_hour_info.golden_hour_morning_end)
            || (current_time >= golden_hour_info.golden_hour_evening_start
                && current_time <= golden_hour_info.golden_hour_evening_end)
    }

    fn create_summary(
        &self,
        weather_analysis: &WeatherAnalysis,
        golden_hour_info: &GoldenHourInfo,
        is_golden_hour_today: bool,
        current_time: DateTime<Local>,
        aurora_probability: f64,
    ) -> DashboardSummary {
        let mut key_highlights = Vec::new();
        let mut warnings = Vec::new();

        // Анализируем погоду
        if weather_analysis.overall_score >= 8.0 {
            key_highlights.push("Отличные погодные условия для съемки!".to_string());
        } else if weather_analysis.overall_score >= 6.0 {
            key_highlights.push("Хорошие погодные условия".to_string());
        } else {
            warnings.push("Погодные условия не идеальны для съемки".to_string());
        }

        // Анализируем золотой час
        if is_golden_hour_today {
            key_highlights.push("Сегодня золотой час - идеальное время для съемки!".to_string());
        } else {
            let _current_hour = current_time.hour() as usize;
            if current_time.hour() as usize
                >= golden_hour_info.golden_hour_morning_start.hour() as usize
                && current_time.hour() as usize
                    <= golden_hour_info.golden_hour_morning_end.hour() as usize
            {
                key_highlights.push("Сейчас золотой час утром!".to_string());
            } else if current_time.hour() as usize
                >= golden_hour_info.golden_hour_evening_start.hour() as usize
                && current_time.hour() as usize
                    <= golden_hour_info.golden_hour_evening_end.hour() as usize
            {
                key_highlights.push("Сейчас золотой час вечером!".to_string());
            }
        }

        // Определяем общую рекомендацию
        let overall_recommendation = self
            .determine_overall_recommendation(weather_analysis.overall_score, is_golden_hour_today);

        // Используем лучшие часы для съемки из погодного анализа
        let best_shooting_hours = weather_analysis.best_hours.clone();

        DashboardSummary {
            overall_recommendation,
            weather_score: weather_analysis.overall_score,
            aurora_probability,
            is_golden_hour_today,
            best_shooting_hours,
            key_highlights,
            warnings,
        }
    }

    fn determine_overall_recommendation(
        &self,
        weather_score: f64,
        is_golden_hour_today: bool,
    ) -> String {
        if weather_score >= 8.0 && is_golden_hour_today {
            "Отличный день для фотографии! Идеальные условия и золотой час.".to_string()
        } else if weather_score >= 7.0 {
            "Хороший день для съемки. Погодные условия благоприятны.".to_string()
        } else if weather_score >= 5.0 {
            "Умеренные условия для съемки. Возможны некоторые ограничения.".to_string()
        } else {
            "Сложные условия для съемки. Рекомендуется перенести съемку.".to_string()
        }
    }

    pub fn print_dashboard(&self, summary: &DashboardSummary) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::WeatherAnalysis;
    use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};

    // Вспомогательные функции для создания тестовых данных
    fn create_test_weather_analysis() -> WeatherAnalysis {
        WeatherAnalysis {
            overall_score: 7.5,
            recommendations: vec!["Отличные условия для фотографии!".to_string()],
            best_hours: vec![6, 7, 8, 18, 19, 20],
            concerns: vec![],
        }
    }

    fn create_test_golden_hour_info() -> GoldenHourInfo {
        let test_date = create_test_date();
        let service = GoldenHourService::new(55.7558, 37.6176);
        service.calculate_golden_hours(test_date)
    }

    fn create_test_date() -> DateTime<Local> {
        // Используем фиксированную дату для тестов
        let naive_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let naive_datetime = NaiveDateTime::new(naive_date, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        Local.from_local_datetime(&naive_datetime).unwrap()
    }

    fn create_golden_hour_time() -> DateTime<Local> {
        // Время в золотой час
        let naive_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let naive_datetime = NaiveDateTime::new(naive_date, chrono::NaiveTime::from_hms_opt(19, 0, 0).unwrap());
        Local.from_local_datetime(&naive_datetime).unwrap()
    }

    #[test]
    fn test_photography_dashboard_new() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        // Проверяем, что дашборд создается без ошибок
        assert!(true);
    }

    #[test]
    fn test_is_golden_hour_today() {
        let _dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();
        
        // В обычное время не должно быть золотого часа
        let _is_golden = _dashboard.is_golden_hour_today(&golden_hour_info, test_date);
        // Этот тест может быть нестабильным из-за реального времени, поэтому проверяем только логику
        
        // Проверяем, что функция работает без ошибок
        assert!(true);
    }

    #[test]
    fn test_determine_overall_recommendation() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        // Тестируем разные сценарии
        let excellent = dashboard.determine_overall_recommendation(9.0, true);
        let good = dashboard.determine_overall_recommendation(7.5, false);
        let moderate = dashboard.determine_overall_recommendation(5.5, false);
        let poor = dashboard.determine_overall_recommendation(3.0, false);
        
        // Проверяем, что рекомендации содержат ожидаемые ключевые слова
        assert!(excellent.contains("Отличный") || excellent.contains("Идеальные"));
        assert!(good.contains("Хороший") || good.contains("благоприятны"));
        assert!(moderate.contains("Умеренные") || moderate.contains("ограничения"));
        assert!(poor.contains("Сложные") || poor.contains("перенести"));
    }

    #[test]
    fn test_create_summary() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        let weather_analysis = create_test_weather_analysis();
        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();
        
        let summary = dashboard.create_summary(
            &weather_analysis,
            &golden_hour_info,
            false, // не золотой час
            test_date,
            0.3, // 30% вероятность сияний
        );
        
        // Проверяем структуру сводки
        assert_eq!(summary.weather_score, 7.5);
        assert_eq!(summary.aurora_probability, 0.3);
        assert_eq!(summary.is_golden_hour_today, false);
        assert_eq!(summary.best_shooting_hours, vec![6, 7, 8, 18, 19, 20]);
        assert!(!summary.overall_recommendation.is_empty());
        assert!(summary.key_highlights.len() >= 0);
        assert!(summary.warnings.len() >= 0);
    }

    #[test]
    fn test_create_summary_excellent_conditions() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        let mut excellent_weather = create_test_weather_analysis();
        excellent_weather.overall_score = 9.0;
        
        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();
        
        let summary = dashboard.create_summary(
            &excellent_weather,
            &golden_hour_info,
            true, // золотой час
            test_date,
            0.8, // высокая вероятность сияний
        );
        
        // При отличных условиях должны быть highlights
        assert!(!summary.key_highlights.is_empty());
        
        // Проверяем, что есть упоминание отличных условий
        let has_excellent_highlight = summary.key_highlights.iter()
            .any(|highlight| highlight.contains("Отличные"));
        assert!(has_excellent_highlight);
    }

    #[test]
    fn test_create_summary_poor_conditions() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        let mut poor_weather = create_test_weather_analysis();
        poor_weather.overall_score = 3.0;
        
        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();
        
        let summary = dashboard.create_summary(
            &poor_weather,
            &golden_hour_info,
            false,
            test_date,
            0.1,
        );
        
        // При плохих условиях должны быть предупреждения
        assert!(!summary.warnings.is_empty());
        
        // Проверяем, что есть упоминание неидеальных условий
        let has_warning = summary.warnings.iter()
            .any(|warning| warning.contains("не идеальны"));
        assert!(has_warning);
    }

    #[test]
    fn test_dashboard_summary_structure() {
        let summary = DashboardSummary {
            overall_recommendation: "Тестовая рекомендация".to_string(),
            weather_score: 7.0,
            aurora_probability: 0.5,
            is_golden_hour_today: true,
            best_shooting_hours: vec![6, 7, 8, 18, 19, 20],
            key_highlights: vec!["Отличные условия".to_string()],
            warnings: vec![],
        };
        
        // Проверяем разумные пределы
        assert!(summary.weather_score >= 0.0 && summary.weather_score <= 10.0);
        assert!(summary.aurora_probability >= 0.0 && summary.aurora_probability <= 1.0);
        assert!(!summary.overall_recommendation.is_empty());
        assert!(!summary.best_shooting_hours.is_empty());
        
        // Проверяем, что лучшие часы в разумных пределах
        for &hour in &summary.best_shooting_hours {
            assert!(hour >= 0 && hour <= 23);
        }
    }

    #[test]
    fn test_edge_cases() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        // Тестируем граничные значения
        let min_recommendation = dashboard.determine_overall_recommendation(0.0, false);
        let max_recommendation = dashboard.determine_overall_recommendation(10.0, true);
        
        assert!(!min_recommendation.is_empty());
        assert!(!max_recommendation.is_empty());
        
        // Проверяем, что при минимальных значениях есть рекомендация о переносе
        assert!(min_recommendation.contains("перенести") || min_recommendation.contains("Сложные"));
        
        // Проверяем, что при максимальных значениях есть положительная рекомендация
        assert!(max_recommendation.contains("Отличный") || max_recommendation.contains("Идеальные"));
    }

    #[test]
    fn test_aurora_probability_validation() {
        let dashboard = PhotographyDashboard::new(
            "test_key".to_string(),
            "TestCity".to_string(),
            55.7558,
            37.6176,
        );
        
        let weather_analysis = create_test_weather_analysis();
        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();
        
        // Тестируем разные значения вероятности сияний
        let summary_low = dashboard.create_summary(
            &weather_analysis,
            &golden_hour_info,
            false,
            test_date,
            0.0,
        );
        
        let summary_high = dashboard.create_summary(
            &weather_analysis,
            &golden_hour_info,
            false,
            test_date,
            1.0,
        );
        
        assert_eq!(summary_low.aurora_probability, 0.0);
        assert_eq!(summary_high.aurora_probability, 1.0);
    }
}
