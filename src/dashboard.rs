use crate::golden_hour::{GoldenHourInfo, GoldenHourService};
use crate::weather::{analyze_weather_for_photography, WeatherAnalysis, WeatherService};
use chrono::{DateTime, Local, Timelike};
use colored::*;
use crate::solar::predict_aurora;

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
                eprintln!(
                    "{}",
                    "💡 РЕШЕНИЕ: Проверьте интернет-соединение".yellow()
                );
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
        println!("   Вероятность северных сияний: {:.0}%", summary.aurora_probability * 100.0);
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
