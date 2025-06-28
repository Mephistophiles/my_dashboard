//! # Photography Tips Module
//!
//! Модуль для генерации персонализированных советов по фотографии.
//! Предоставляет рекомендации по оборудованию, настройкам камеры,
//! выбору локаций и технике съемки на основе текущих условий.
//!
//! ## Основные компоненты
//!
//! - [`PhotographyTipsService`] - Сервис для генерации советов
//! - [`PhotographyTips`] - Структура с рекомендациями
//!
//! ## Пример использования
//!
//! ```rust
//! use my_dashboard::photography_tips::PhotographyTipsService;
//!
//! // Создаем сервис советов
//! let service = PhotographyTipsService::new();
//!
//! // Получаем персонализированные советы
//! let tips = service.get_tips_for_weather(8.5, true, 0.7);
//!
//! println!("Рекомендации по оборудованию:");
//! for tip in &tips.equipment_recommendations {
//!     println!("- {}", tip);
//! }
//!
//! // Получаем общие рекомендации
//! let general_tips = service.get_general_recommendations();
//! for tip in &general_tips {
//!     println!("- {}", tip);
//! }
//! ```

/// Структура с рекомендациями по фотографии
///
/// Содержит персонализированные советы по оборудованию, съемке,
/// выбору локаций и техническим настройкам камеры.
#[derive(Debug)]
pub struct PhotographyTips {
    /// Рекомендации по необходимому оборудованию
    pub equipment_recommendations: Vec<String>,
    /// Советы по технике съемки
    pub shooting_tips: Vec<String>,
    /// Предложения по выбору локаций
    pub location_suggestions: Vec<String>,
    /// Рекомендуемые технические настройки камеры
    pub technical_settings: Vec<String>,
}

/// Сервис для генерации советов по фотографии
///
/// Анализирует текущие условия (погода, золотой час, северные сияния)
/// и генерирует персонализированные рекомендации для фотографов.
pub struct PhotographyTipsService;

impl PhotographyTipsService {
    /// Создает новый экземпляр сервиса советов
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::photography_tips::PhotographyTipsService;
    ///
    /// let service = PhotographyTipsService::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Генерирует персонализированные советы на основе текущих условий
    ///
    /// # Аргументы
    ///
    /// * `weather_score` - Оценка погодных условий (0-10)
    /// * `is_golden_hour` - Сейчас ли золотой час
    /// * `aurora_probability` - Вероятность северных сияний (0-1)
    ///
    /// # Возвращает
    ///
    /// `PhotographyTips` - Структура с рекомендациями
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::photography_tips::PhotographyTipsService;
    ///
    /// let service = PhotographyTipsService::new();
    /// let tips = service.get_tips_for_weather(8.5, true, 0.7);
    ///
    /// // Проверяем рекомендации по оборудованию
    /// assert!(!tips.equipment_recommendations.is_empty());
    /// ```
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

    /// Возвращает общие рекомендации по фотографии
    ///
    /// Содержит универсальные советы, которые применимы в любых условиях
    /// и помогут улучшить качество съемки.
    ///
    /// # Возвращает
    ///
    /// `Vec<String>` - Список общих рекомендаций
    ///
    /// # Пример
    ///
    /// ```rust
    /// use my_dashboard::photography_tips::PhotographyTipsService;
    ///
    /// let service = PhotographyTipsService::new();
    /// let general_tips = service.get_general_recommendations();
    ///
    /// for tip in &general_tips {
    ///     println!("- {}", tip);
    /// }
    /// ```
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

impl Default for PhotographyTipsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_tips_for_weather_good_conditions() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(8.0, true, 0.7);

        // При хороших условиях должны быть рекомендации
        assert!(!tips.equipment_recommendations.is_empty());
        assert!(!tips.shooting_tips.is_empty());
        assert!(!tips.location_suggestions.is_empty());
        assert!(!tips.technical_settings.is_empty());
    }

    #[test]
    fn test_get_tips_for_weather_bad_conditions() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(3.0, false, 0.1);

        // При плохих условиях должны быть рекомендации по защите
        let has_protection_tips = tips
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("защиту") || tip.contains("штатив"));
        assert!(has_protection_tips);

        // При низкой вероятности сияний не должно быть рекомендаций по ним
        let has_aurora_tips = tips
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("широкоугольный"));
        assert!(!has_aurora_tips);
    }

    #[test]
    fn test_get_tips_for_weather_golden_hour_only() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(6.0, true, 0.1);

        // Проверяем рекомендации для золотого часа
        let has_golden_hour_equipment = tips
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("фильтр"));
        assert!(has_golden_hour_equipment);

        let has_golden_hour_shooting = tips
            .shooting_tips
            .iter()
            .any(|tip| tip.contains("теплые тона"));
        assert!(has_golden_hour_shooting);

        let has_golden_hour_locations = tips
            .location_suggestions
            .iter()
            .any(|tip| tip.contains("парки") || tip.contains("набережные"));
        assert!(has_golden_hour_locations);

        let has_golden_hour_settings = tips
            .technical_settings
            .iter()
            .any(|tip| tip.contains("f/8") || tip.contains("f/16"));
        assert!(has_golden_hour_settings);
    }

    #[test]
    fn test_get_tips_for_weather_aurora_only() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(6.0, false, 0.8);

        // Проверяем рекомендации для северных сияний
        let has_aurora_equipment = tips
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("Широкоугольный"));
        assert!(has_aurora_equipment);

        let has_aurora_shooting = tips
            .shooting_tips
            .iter()
            .any(|tip| tip.contains("длинные выдержки"));
        assert!(has_aurora_shooting);

        let has_aurora_locations = tips
            .location_suggestions
            .iter()
            .any(|tip| tip.contains("город") || tip.contains("открытые пространства"));
        assert!(has_aurora_locations);

        let has_aurora_settings = tips
            .technical_settings
            .iter()
            .any(|tip| tip.contains("15-30 секунд"));
        assert!(has_aurora_settings);
    }

    #[test]
    fn test_get_tips_for_weather_excellent_conditions() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(9.0, false, 0.1);

        // При отличных условиях должны быть соответствующие советы (weather_score >= 7.0)
        let has_excellent_tips = tips
            .shooting_tips
            .iter()
            .any(|tip| tip.contains("Отличные условия"));
        assert!(has_excellent_tips);

        let has_experiment_tips = tips
            .shooting_tips
            .iter()
            .any(|tip| tip.contains("экспериментируйте"));
        assert!(has_experiment_tips);
    }

    #[test]
    fn test_get_tips_for_weather_all_conditions() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(8.0, true, 0.8);

        // При всех благоприятных условиях должно быть много рекомендаций
        assert!(tips.equipment_recommendations.len() >= 4);
        assert!(tips.shooting_tips.len() >= 4);
        assert!(tips.location_suggestions.len() >= 3);
        assert!(tips.technical_settings.len() >= 4);

        // Проверяем наличие всех типов рекомендаций
        let has_golden_hour_tips = tips
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("фильтр"));
        let has_aurora_tips = tips
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("Широкоугольный"));
        let has_excellent_tips = tips
            .shooting_tips
            .iter()
            .any(|tip| tip.contains("Отличные условия"));

        assert!(has_golden_hour_tips);
        assert!(has_aurora_tips);
        assert!(has_excellent_tips);
    }

    #[test]
    fn test_get_general_recommendations() {
        let service = PhotographyTipsService::new();
        let recommendations = service.get_general_recommendations();

        // Должно быть 5 общих рекомендаций
        assert_eq!(recommendations.len(), 5);

        // Проверяем наличие ключевых рекомендаций
        let has_weather_check = recommendations
            .iter()
            .any(|rec| rec.contains("прогноз погоды"));
        let has_planning = recommendations.iter().any(|rec| rec.contains("Планируйте"));
        let has_safety = recommendations
            .iter()
            .any(|rec| rec.contains("безопасности"));

        assert!(has_weather_check);
        assert!(has_planning);
        assert!(has_safety);
    }

    #[test]
    fn test_photography_tips_structure() {
        let service = PhotographyTipsService::new();
        // Используем параметры, которые гарантированно добавят рекомендации
        let tips = service.get_tips_for_weather(4.0, true, 0.6);

        // Проверяем структуру объекта
        assert!(!tips.equipment_recommendations.is_empty());
        assert!(!tips.shooting_tips.is_empty());
        assert!(!tips.location_suggestions.is_empty());
        assert!(!tips.technical_settings.is_empty());

        // Проверяем, что все рекомендации не пустые
        for tip in &tips.equipment_recommendations {
            assert!(!tip.is_empty());
        }
        for tip in &tips.shooting_tips {
            assert!(!tip.is_empty());
        }
        for tip in &tips.location_suggestions {
            assert!(!tip.is_empty());
        }
        for tip in &tips.technical_settings {
            assert!(!tip.is_empty());
        }
    }

    #[test]
    fn test_edge_cases() {
        let service = PhotographyTipsService::new();

        // Тестируем граничные значения
        let tips_min = service.get_tips_for_weather(0.0, false, 0.0);
        let tips_max = service.get_tips_for_weather(10.0, true, 1.0);

        // При минимальных значениях должны быть рекомендации по защите
        let has_protection_min = tips_min
            .equipment_recommendations
            .iter()
            .any(|tip| tip.contains("защиту") || tip.contains("штатив"));
        assert!(has_protection_min);

        // При максимальных значениях должно быть много рекомендаций
        assert!(!tips_max.equipment_recommendations.is_empty());
        assert!(!tips_max.shooting_tips.is_empty());
        assert!(!tips_max.location_suggestions.is_empty());
        assert!(!tips_max.technical_settings.is_empty());
    }

    #[test]
    fn test_technical_settings_consistency() {
        let service = PhotographyTipsService::new();
        let tips = service.get_tips_for_weather(7.0, true, 0.6);

        // Проверяем, что технические настройки содержат разумные значения
        for setting in &tips.technical_settings {
            // Проверяем ISO
            if setting.contains("ISO") {
                assert!(
                    setting.contains("100")
                        || setting.contains("400")
                        || setting.contains("800")
                        || setting.contains("3200")
                );
            }

            // Проверяем диафрагму
            if setting.contains("f/") {
                assert!(
                    setting.contains("f/2.8")
                        || setting.contains("f/4")
                        || setting.contains("f/8")
                        || setting.contains("f/16")
                );
            }

            // Проверяем выдержку
            if setting.contains("секунд") {
                assert!(
                    setting.contains("1/60")
                        || setting.contains("1/250")
                        || setting.contains("15-30")
                );
            }
        }
    }
}
