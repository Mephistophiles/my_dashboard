#!/usr/bin/env bash

# Скрипт для запуска дашборда фотографа
# Использование: ./run.sh [город] [широта] [долгота] [api_key]

echo "📸 Запуск дашборда для фотографов"
echo "=================================="

# Параметры по умолчанию
CITY=${1:-"Москва"}
LATITUDE=${2:-"55.7558"}
LONGITUDE=${3:-"37.6176"}
OPENWEATHER_API_KEY=${4:-"demo_key"}

echo "📍 Локация: $CITY ($LATITUDE, $LONGITUDE)"
echo "🔑 API ключ: ${OPENWEATHER_API_KEY:0:8}..." # Показываем только первые 8 символов для безопасности

# Проверяем DEMO режим
if [ "$OPENWEATHER_API_KEY" = "demo_key" ] || [ "$DEMO_MODE" = "true" ]; then
    echo "🎭 DEMO режим: используются демонстрационные данные"
    export DEMO_MODE=true
else
    echo "🌐 Production режим: используются реальные данные API"
    export DEMO_MODE=false
fi

echo ""

# Проверяем, есть ли API ключ в переменных окружения
if [ -z "$4" ] && [ -n "$OPENWEATHER_API_KEY" ]; then
    echo "ℹ️  Используется API ключ из переменной окружения OPENWEATHER_API_KEY"
fi

# Установка переменных окружения
export CITY="$CITY"
export LATITUDE="$LATITUDE"
export LONGITUDE="$LONGITUDE"
export OPENWEATHER_API_KEY="$OPENWEATHER_API_KEY"

# Проверяем, что Cargo установлен
if ! command -v cargo &> /dev/null; then
    echo "❌ Ошибка: Cargo не найден. Убедитесь, что Rust установлен."
    exit 1
fi

# Проверяем, что проект собран
if [ ! -d "target" ]; then
    echo "🔨 Первая сборка проекта..."
    cargo build --release
fi

echo "🚀 Запуск дашборда..."
echo ""

# Запуск приложения
cargo run

echo ""
echo "✅ Дашборд завершен" 
