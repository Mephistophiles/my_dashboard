use crate::golden_hour::{GoldenHourInfo, GoldenHourService};
use crate::photography_tips::{PhotographyTips, PhotographyTipsService};
use crate::solar::{AuroraForecast, SolarService};
use crate::weather::{
    analyze_weather_for_photography, WeatherAnalysis, WeatherForecast, WeatherService,
};
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
    solar_service: SolarService,
    golden_hour_service: GoldenHourService,
    tips_service: PhotographyTipsService,
}

impl PhotographyDashboard {
    pub fn new(api_key: String, city: String, latitude: f64, longitude: f64) -> Self {
        Self {
            weather_service: WeatherService::new(api_key, city),
            solar_service: SolarService::new(),
            golden_hour_service: GoldenHourService::new(latitude, longitude),
            tips_service: PhotographyTipsService::new(),
        }
    }

    pub async fn generate_dashboard(&self) -> Result<DashboardSummary, Box<dyn std::error::Error>> {
        let current_time = Local::now();

        // Получаем данные о погоде
        let weather_forecast = self.weather_service.get_weather_forecast().await?;
        let weather_analysis = analyze_weather_for_photography(&weather_forecast);

        // Получаем данные о солнечной активности
        let solar_wind_data = self.solar_service.get_solar_wind_data().await?;
        let geomagnetic_data = self.solar_service.get_geomagnetic_data().await?;
        let aurora_forecast = self
            .solar_service
            .predict_aurora(&solar_wind_data, &geomagnetic_data);

        // Получаем информацию о золотом часе
        let golden_hour_info = self
            .golden_hour_service
            .calculate_golden_hours(current_time);

        // Определяем, есть ли золотой час сегодня
        let is_golden_hour_today = self.is_golden_hour_today(&golden_hour_info, current_time);

        // Создаем общую сводку
        let summary = self.create_summary(
            &weather_analysis,
            &aurora_forecast,
            &golden_hour_info,
            is_golden_hour_today,
            current_time,
        );

        Ok(summary)
    }

    fn is_golden_hour_today(
        &self,
        golden_hour_info: &GoldenHourInfo,
        current_time: DateTime<Local>,
    ) -> bool {
        let current_hour = current_time.hour() as usize;

        // Проверяем, попадает ли текущее время в золотой час
        (current_time >= golden_hour_info.golden_hour_morning_start
            && current_time <= golden_hour_info.golden_hour_morning_end)
            || (current_time >= golden_hour_info.golden_hour_evening_start
                && current_time <= golden_hour_info.golden_hour_evening_end)
    }

    fn create_summary(
        &self,
        weather_analysis: &WeatherAnalysis,
        aurora_forecast: &AuroraForecast,
        golden_hour_info: &GoldenHourInfo,
        is_golden_hour_today: bool,
        current_time: DateTime<Local>,
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

        // Анализируем северные сияния
        if aurora_forecast.visibility_probability > 0.7 {
            key_highlights.push("Высокая вероятность северных сияний!".to_string());
        } else if aurora_forecast.visibility_probability > 0.4 {
            key_highlights.push("Умеренная вероятность северных сияний".to_string());
        }

        // Анализируем золотой час
        if is_golden_hour_today {
            key_highlights.push("Сегодня золотой час - идеальное время для съемки!".to_string());
        } else {
            let current_hour = current_time.hour() as usize;
            if current_hour >= golden_hour_info.golden_hour_morning_start.hour() as usize
                && current_hour <= golden_hour_info.golden_hour_morning_end.hour() as usize
            {
                key_highlights.push("Сейчас золотой час утром!".to_string());
            } else if current_hour >= golden_hour_info.golden_hour_evening_start.hour() as usize
                && current_hour <= golden_hour_info.golden_hour_evening_end.hour() as usize
            {
                key_highlights.push("Сейчас золотой час вечером!".to_string());
            }
        }

        // Определяем общую рекомендацию
        let overall_recommendation = self.determine_overall_recommendation(
            weather_analysis.overall_score,
            aurora_forecast.visibility_probability,
            is_golden_hour_today,
        );

        // Объединяем лучшие часы для съемки
        let mut best_shooting_hours = weather_analysis.best_hours.clone();
        best_shooting_hours.extend(&aurora_forecast.best_viewing_hours);
        best_shooting_hours.sort();
        best_shooting_hours.dedup();

        DashboardSummary {
            overall_recommendation,
            weather_score: weather_analysis.overall_score,
            aurora_probability: aurora_forecast.visibility_probability,
            is_golden_hour_today,
            best_shooting_hours,
            key_highlights,
            warnings,
        }
    }

    fn determine_overall_recommendation(
        &self,
        weather_score: f64,
        aurora_probability: f64,
        is_golden_hour_today: bool,
    ) -> String {
        let mut score = 0.0;

        // Влияние погоды (40% веса)
        score += weather_score * 0.4;

        // Влияние северных сияний (30% веса)
        score += aurora_probability * 10.0 * 0.3;

        // Влияние золотого часа (30% веса)
        if is_golden_hour_today {
            score += 10.0 * 0.3;
        }

        match score {
            s if s >= 8.0 => "🚀 ОТЛИЧНО! Сегодня идеальный день для фотографии!".to_string(),
            s if s >= 6.0 => "✅ ХОРОШО! Условия подходят для съемки".to_string(),
            s if s >= 4.0 => "⚠️ УМЕРЕННО. Условия приемлемые, но не идеальные".to_string(),
            _ => "❌ НЕ РЕКОМЕНДУЕТСЯ. Лучше перенести съемку на другой день".to_string(),
        }
    }

    pub fn print_dashboard(&self, summary: &DashboardSummary) {
        println!("\n{}", "=".repeat(60));
        println!(
            "{}",
            "📸 ДАШБОРД ДЛЯ ФОТОГРАФОВ 📸".bold().white().on_blue()
        );
        println!("{}", "=".repeat(60));

        println!("\n{}", summary.overall_recommendation.bold());

        if !summary.key_highlights.is_empty() {
            println!("\n{}:", "Ключевые моменты".bold().green());
            for highlight in &summary.key_highlights {
                println!("  ✨ {}", highlight);
            }
        }

        if !summary.warnings.is_empty() {
            println!("\n{}:", "Предупреждения".bold().red());
            for warning in &summary.warnings {
                println!("  ⚠ {}", warning);
            }
        }

        println!(
            "\n{}: {:.1}/10",
            "Оценка погоды".bold(),
            summary.weather_score
        );
        println!(
            "{}: {:.1}%",
            "Вероятность северных сияний".bold(),
            summary.aurora_probability * 100.0
        );
        println!(
            "{}: {}",
            "Золотой час сегодня".bold(),
            if summary.is_golden_hour_today {
                "Да"
            } else {
                "Нет"
            }
        );

        if !summary.best_shooting_hours.is_empty() {
            println!("\n{}:", "Лучшие часы для съемки".bold().yellow());
            for hour in &summary.best_shooting_hours {
                println!("  🕐 {}:00", hour);
            }
        }

        println!("\n{}", "=".repeat(60));
    }
}
