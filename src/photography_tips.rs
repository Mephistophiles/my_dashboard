use colored::*;

#[derive(Debug)]
pub struct PhotographyTips {
    pub equipment_recommendations: Vec<String>,
    pub shooting_tips: Vec<String>,
    pub location_suggestions: Vec<String>,
    pub technical_settings: Vec<String>,
}

pub struct PhotographyTipsService;

impl PhotographyTipsService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_tips_for_weather(
        &self,
        weather_score: f64,
        is_golden_hour: bool,
        aurora_probability: f64,
    ) -> PhotographyTips {
        let mut tips = PhotographyTips {
            equipment_recommendations: Vec::new(),
            shooting_tips: Vec::new(),
            location_suggestions: Vec::new(),
            technical_settings: Vec::new(),
        };

        // Рекомендации по оборудованию
        if weather_score < 5.0 {
            tips.equipment_recommendations
                .push("Возьмите защиту от дождя для камеры".to_string());
            tips.equipment_recommendations
                .push("Используйте штатив для стабилизации".to_string());
        }

        if aurora_probability > 0.5 {
            tips.equipment_recommendations
                .push("Широкоугольный объектив для северных сияний".to_string());
            tips.equipment_recommendations
                .push("Удаленный спуск затвора".to_string());
            tips.equipment_recommendations
                .push("Теплая одежда - съемка может занять время".to_string());
        }

        if is_golden_hour {
            tips.equipment_recommendations
                .push("Градиентные фильтры для баланса экспозиции".to_string());
            tips.equipment_recommendations
                .push("Поляризационный фильтр".to_string());
        }

        // Советы по съемке
        if is_golden_hour {
            tips.shooting_tips
                .push("Используйте теплые тона для создания атмосферы".to_string());
            tips.shooting_tips
                .push("Экспериментируйте с силуэтами".to_string());
            tips.shooting_tips
                .push("Ищите отражающие поверхности (вода, стекло)".to_string());
        }

        if aurora_probability > 0.5 {
            tips.shooting_tips
                .push("Используйте длинные выдержки (15-30 секунд)".to_string());
            tips.shooting_tips
                .push("Фокусируйтесь на бесконечность".to_string());
            tips.shooting_tips
                .push("Снимайте в RAW формате".to_string());
        }

        if weather_score >= 7.0 {
            tips.shooting_tips
                .push("Отличные условия - экспериментируйте с композицией".to_string());
            tips.shooting_tips
                .push("Попробуйте разные ракурсы".to_string());
        }

        // Рекомендации по локациям
        if aurora_probability > 0.5 {
            tips.location_suggestions
                .push("Отправляйтесь за город, подальше от светового загрязнения".to_string());
            tips.location_suggestions
                .push("Ищите открытые пространства с хорошим обзором севера".to_string());
        }

        if is_golden_hour {
            tips.location_suggestions
                .push("Парки и природные зоны".to_string());
            tips.location_suggestions
                .push("Городские набережные".to_string());
            tips.location_suggestions
                .push("Смотровые площадки".to_string());
        }

        // Технические настройки
        if is_golden_hour {
            tips.technical_settings.push("ISO: 100-400".to_string());
            tips.technical_settings
                .push("Диафрагма: f/8-f/16 для пейзажей".to_string());
            tips.technical_settings
                .push("Выдержка: 1/60 - 1/250 секунды".to_string());
        }

        if aurora_probability > 0.5 {
            tips.technical_settings.push("ISO: 800-3200".to_string());
            tips.technical_settings
                .push("Диафрагма: f/2.8-f/4".to_string());
            tips.technical_settings
                .push("Выдержка: 15-30 секунд".to_string());
            tips.technical_settings
                .push("Баланс белого: 3500-4500K".to_string());
        }

        tips
    }

    pub fn get_general_recommendations(&self) -> Vec<String> {
        vec![
            "Всегда проверяйте прогноз погоды перед съемкой".to_string(),
            "Планируйте локации заранее".to_string(),
            "Берите запасные батареи и карты памяти".to_string(),
            "Изучите правила съемки в выбранных местах".to_string(),
            "Не забудьте о безопасности - особенно при съемке в дикой природе".to_string(),
        ]
    }
}

pub fn print_photography_tips(tips: &PhotographyTips) {
    println!("\n{}", "=== СОВЕТЫ ДЛЯ ФОТОГРАФОВ ===".bold().magenta());

    if !tips.equipment_recommendations.is_empty() {
        println!("\n{}:", "Рекомендуемое оборудование".bold().cyan());
        for rec in &tips.equipment_recommendations {
            println!("  📷 {}", rec);
        }
    }

    if !tips.shooting_tips.is_empty() {
        println!("\n{}:", "Советы по съемке".bold().green());
        for tip in &tips.shooting_tips {
            println!("  💡 {}", tip);
        }
    }

    if !tips.location_suggestions.is_empty() {
        println!("\n{}:", "Рекомендуемые локации".bold().blue());
        for location in &tips.location_suggestions {
            println!("  📍 {}", location);
        }
    }

    if !tips.technical_settings.is_empty() {
        println!("\n{}:", "Технические настройки".bold().yellow());
        for setting in &tips.technical_settings {
            println!("  ⚙ {}", setting);
        }
    }
}
