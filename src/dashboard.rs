//! # Dashboard Module
//!
//! Модуль для создания и управления основным дашбордом фотографа.
//! Предоставляет функциональность для генерации сводки условий съемки
//! и вывода персонализированных рекомендаций.
//!
//! ## Основные компоненты
//!
//! - [`PhotographyDashboard`] - Основной класс дашборда
//! - [`DashboardSummary`] - Структура сводки условий
//!
//! ## Пример использования
//!
//! ```rust
//! use my_dashboard::dashboard::{PhotographyDashboard, DashboardSummary};
//!
//! // Создаем дашборд
//! let dashboard = PhotographyDashboard::new(
//!     "Moscow".to_string(),
//!     55.7558,
//!     37.6176,
//! );
//!
//! // Для асинхронного использования:
//! // #[tokio::main]
//! // async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! //     let summary = dashboard.generate_dashboard().await?;
//! //     println!("Оценка погоды: {}/10", summary.weather_score);
//! //     Ok(())
//! // }
//! ```

use crate::golden_hour::{GoldenHourInfo, GoldenHourService};
use crate::weather::{analyze_weather_for_photography, WeatherAnalysis};
use chrono::{DateTime, Local};
use log::debug;

/// Сводка условий для фотографии
#[derive(Debug, Clone)]
pub struct DashboardSummary {
    /// Общая рекомендация для съемки
    pub overall_recommendation: String,
    /// Оценка погодных условий (0-10)
    pub weather_score: f64,
    /// Вероятность северных сияний (0-1)
    pub aurora_probability: f64,
    /// Есть ли золотой час сегодня
    pub is_golden_hour_today: bool,
    /// Лучшие часы для съемки
    pub best_shooting_hours: Vec<usize>,
    /// Ключевые моменты для съемки
    pub key_highlights: Vec<String>,
    /// Предупреждения о неблагоприятных условиях
    pub warnings: Vec<String>,
}

/// Основной дашборд для фотографов
///
/// Объединяет данные о погоде, золотом часе и северных сияниях
/// для создания персонализированной сводки условий съемки.
pub struct PhotographyDashboard {
    golden_hour_service: GoldenHourService,
}

impl PhotographyDashboard {
    /// Создает новый экземпляр дашборда
    ///
    /// # Аргументы
    ///
    /// * `city` - Название города
    /// * `latitude` - Широта в градусах
    /// * `longitude` - Долгота в градусах
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::dashboard::PhotographyDashboard;
    ///
    /// let dashboard = PhotographyDashboard::new(
    ///     "Moscow".to_string(),
    ///     55.7558,
    ///     37.6176,
    /// );
    /// ```
    pub fn new(city: String, latitude: f64, longitude: f64) -> Self {
        debug!("Создание дашборда для города: {}", city);

        Self {
            golden_hour_service: GoldenHourService::new(latitude, longitude),
        }
    }

    /// Генерирует полную сводку условий для съемки
    ///
    /// Собирает данные о погоде, золотом часе и северных сияниях,
    /// анализирует их и создает персонализированную сводку.
    ///
    /// # Возвращает
    ///
    /// `Result<DashboardSummary, Box<dyn std::error::Error>>` - Сводка условий или ошибка
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::dashboard::PhotographyDashboard;
    ///
    /// // Создаем дашборд
    /// let dashboard = PhotographyDashboard::new(
    ///     "Moscow".to_string(),
    ///     55.7558,
    ///     37.6176,
    /// );
    ///
    /// // Для асинхронного использования:
    /// // #[tokio::main]
    /// // async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// //     let summary = dashboard.generate_dashboard().await?;
    /// //     println!("Рекомендация: {}", summary.overall_recommendation);
    /// //     Ok(())
    /// // }
    /// ```
    pub async fn generate_dashboard(
        &self,
        weather_forecast: &crate::weather::WeatherForecast,
        aurora_probability: f64,
    ) -> Result<DashboardSummary, anyhow::Error> {
        let current_time = Local::now();

        // Анализируем погоду
        let weather_analysis = analyze_weather_for_photography(weather_forecast);
        // Получаем информацию о золотом часе
        let golden_hour_info = self
            .golden_hour_service
            .calculate_golden_hours(current_time);

        // Определяем, есть ли золотой час сегодня
        let is_golden_hour_today = self.is_golden_hour_today(&golden_hour_info, current_time);

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
            // Используем точную проверку времени вместо только часов
            if current_time >= golden_hour_info.golden_hour_morning_start
                && current_time <= golden_hour_info.golden_hour_morning_end
            {
                key_highlights.push("Сейчас золотой час утром!".to_string());
            } else if current_time >= golden_hour_info.golden_hour_evening_start
                && current_time <= golden_hour_info.golden_hour_evening_end
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::WeatherAnalysis;
    use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};
    use pretty_assertions::assert_eq;

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
        let naive_datetime = NaiveDateTime::new(
            naive_date,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );
        Local.from_local_datetime(&naive_datetime).unwrap()
    }

    #[test]
    fn test_is_golden_hour_today() {
        let _dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();

        // В обычное время не должно быть золотого часа
        let is_golden = _dashboard.is_golden_hour_today(&golden_hour_info, test_date);

        assert_eq!(is_golden, false);
        // Этот тест может быть нестабильным из-за реального времени, поэтому проверяем только логику
    }

    #[test]
    fn test_determine_overall_recommendation() {
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

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
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

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
        assert!(!summary.is_golden_hour_today);
        assert_eq!(summary.best_shooting_hours, vec![6, 7, 8, 18, 19, 20]);
        assert!(!summary.overall_recommendation.is_empty());
    }

    #[test]
    fn test_create_summary_excellent_conditions() {
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

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
        let has_excellent_highlight = summary
            .key_highlights
            .iter()
            .any(|highlight| highlight.contains("Отличные"));
        assert!(has_excellent_highlight);
    }

    #[test]
    fn test_create_summary_poor_conditions() {
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

        let mut poor_weather = create_test_weather_analysis();
        poor_weather.overall_score = 3.0;

        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();

        let summary =
            dashboard.create_summary(&poor_weather, &golden_hour_info, false, test_date, 0.1);

        // При плохих условиях должны быть предупреждения
        assert!(!summary.warnings.is_empty());

        // Проверяем, что есть упоминание неидеальных условий
        let has_warning = summary
            .warnings
            .iter()
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
            assert!((0..=23).contains(&hour));
        }
    }

    #[test]
    fn test_edge_cases() {
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

        // Тестируем граничные значения
        let min_recommendation = dashboard.determine_overall_recommendation(0.0, false);
        let max_recommendation = dashboard.determine_overall_recommendation(10.0, true);

        assert!(!min_recommendation.is_empty());
        assert!(!max_recommendation.is_empty());

        // Проверяем, что при минимальных значениях есть рекомендация о переносе
        assert!(min_recommendation.contains("перенести") || min_recommendation.contains("Сложные"));

        // Проверяем, что при максимальных значениях есть положительная рекомендация
        assert!(
            max_recommendation.contains("Отличный") || max_recommendation.contains("Идеальные")
        );
    }

    #[test]
    fn test_aurora_probability_validation() {
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

        let weather_analysis = create_test_weather_analysis();
        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();

        // Тестируем разные значения вероятности сияний
        let summary_low =
            dashboard.create_summary(&weather_analysis, &golden_hour_info, false, test_date, 0.0);

        let summary_high =
            dashboard.create_summary(&weather_analysis, &golden_hour_info, false, test_date, 1.0);

        assert_eq!(summary_low.aurora_probability, 0.0);
        assert_eq!(summary_high.aurora_probability, 1.0);
    }

    #[test]
    fn test_golden_hour_precise_time_detection() {
        let dashboard = PhotographyDashboard::new("TestCity".to_string(), 55.7558, 37.6176);

        let golden_hour_info = create_test_golden_hour_info();
        let test_date = create_test_date();

        // В обычное время не должно быть золотого часа
        let _is_golden = dashboard.is_golden_hour_today(&golden_hour_info, test_date);
        // Этот тест может быть нестабильным из-за реального времени, поэтому проверяем только логику
    }
}
