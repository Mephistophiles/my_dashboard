# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test coverage improvements (70%+ target)
- Additional unit tests for main.rs, solar.rs, and weather.rs modules
- Edge case testing for all data structures and functions
- Integration tests for error handling scenarios
- Documentation updates for code coverage requirements

### Changed
- Updated documentation with code coverage requirements (>70%)
- Enhanced CHANGELOG format to follow Keep a Changelog standard
- Improved test structure and organization
- Added comprehensive edge case testing

### Fixed
- Test compilation issues in solar.rs (missing Datelike and Timelike imports)
- Dead code warnings by removing all #[allow(dead_code)] attributes
- Unused function and field warnings
- Test failures in photography_tips module

## [0.4.0] - 2025-06-15

### Added
- Photography dashboard functionality with comprehensive analysis
- Golden hour calculation service with sunrise/sunset times
- Solar data integration with aurora prediction
- Photography tips service with personalized recommendations
- Enhanced error handling and logging

### Changed
- Refactored aurora activity calculation to use dedicated function
- Improved golden hour information display with current lighting conditions
- Enhanced weather analysis algorithms
- Updated API integration patterns

### Fixed
- All #[allow(dead_code)] attributes removed
- Test compilation issues resolved
- Unused code eliminated or made functional
- Code coverage improvements

## [0.3.0] - 2025-06-15

### Changed
- **Полностью убран хардкод из production режима** - все данные теперь получаются из реальных API
- **Улучшена обработка ошибок** - при недоступности данных программа завершается с ошибкой вместо использования хардкода
- **Добавлена интеграция модуля советов по фотографии** - теперь использует реальные данные о погоде и золотом часе
- **Добавлен вывод вероятности северных сияний** - ранее неиспользуемая функция теперь интегрирована в дашборд

### Added
- Реальное получение вероятности северных сияний в main.rs (вместо хардкода 0.0)
- Правильная обработка ошибок при получении данных о северных сияниях
- Опциональные поля для недоступных данных (magnetic_field, solar_radiation)
- Персонализированные советы по фотографии на основе реальных данных

### Fixed
- Хардкод `aurora_probability = 0.0` в main.rs
- Fallback на 0.0 при ошибках в dashboard.rs
- Хардкод `magnetic_field: 0.0` и `solar_radiation: 0.0` в solar.rs
- Неиспользуемые импорты и предупреждения компилятора
- Форматирование кода согласно стандартам Rust

### Removed
- Все хардкодные значения в production режиме
- Fallback на моковые данные при ошибках API

## [0.2.0] - 2025-06-15

### Changed
- **Убрано использование случайных данных в production режиме** - теперь используются только реальные данные от OpenWeatherMap API
- **Добавлен DEMO режим** - для тестирования и демонстрации можно использовать `DEMO_MODE=true`
- **Исправлено название переменной окружения** - `WEATHER_API_KEY` → `OPENWEATHER_API_KEY`
- **Улучшена обработка ошибок** - при ошибках API программа завершается с понятными сообщениями
- **Убрана зависимость `rand`** - больше не используется в production режиме

### Added
- Переменная окружения `DEMO_MODE` для включения демонстрационного режима
- Детерминированные суточные циклы для прогноза погоды (без случайности)
- Подробная документация по DEMO режиму
- Обновленные примеры использования

### Fixed
- Все предупреждения компилятора (кроме неиспользуемых импортов)
- Документация в README.md, USAGE.md, env.example
- Скрипт run.sh для поддержки DEMO режима

### Removed
- Зависимость `rand = "0.8"` из Cargo.toml
- Случайные вариации в прогнозе погоды

## [0.1.0] - 2025-06-15

### Added
- Базовый функционал дашборда для фотографов
- Модули для погоды, солнечной активности, золотого часа
- Консольный интерфейс с цветным выводом
- Поддержка OpenWeatherMap API 