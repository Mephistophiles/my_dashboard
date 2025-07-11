# 📖 Руководство по использованию v0.3.0

## 🚀 Быстрый запуск

### Базовое использование
```bash
./run.sh
```
Запускает дашборд с параметрами по умолчанию (Москва, 55.7558, 37.6176, demo_key)

### С указанием города
```bash
./run.sh "Санкт-Петербург"
```

### С указанием координат
```bash
./run.sh "Новосибирск" 55.0084 82.9357
```

### С API ключом
```bash
./run.sh "Москва" 55.7558 37.6176 "your_api_key_here"
```

## 🔑 Настройка API ключа

### Способ 1: Переменная окружения
```bash
export OPENWEATHER_API_KEY="your_api_key_here"
./run.sh
```

### Способ 2: Параметр скрипта
```bash
./run.sh "Москва" 55.7558 37.6176 "your_api_key_here"
```

### Способ 3: Файл .env (рекомендуется)
```bash
# Создайте файл .env в корне проекта
echo "OPENWEATHER_API_KEY=your_api_key_here" > .env
echo "CITY=Москва" >> .env
echo "LATITUDE=55.7558" >> .env
echo "LONGITUDE=37.6176" >> .env
```

## 🎭 DEMO режим

Для тестирования и демонстрации можно использовать DEMO режим:

```bash
# Включить DEMO режим
export DEMO_MODE=true
./run.sh

# Или через .env файл
echo "DEMO_MODE=true" >> .env
./run.sh
```

**Важно:** В DEMO режиме используются демонстрационные данные. Для получения реальных данных о погоде используйте настоящий API ключ OpenWeatherMap.

## 📍 Популярные города

### Россия
```bash
# Москва
./run.sh "Москва" 55.7558 37.6176

# Санкт-Петербург
./run.sh "Санкт-Петербург" 59.9311 30.3609

# Новосибирск
./run.sh "Новосибирск" 55.0084 82.9357

# Екатеринбург
./run.sh "Екатеринбург" 56.8431 60.6454

# Казань
./run.sh "Казань" 55.8304 49.0661
```

### Северные сияния (высокая вероятность)
```bash
# Мурманск
./run.sh "Мурманск" 68.9792 33.0925

# Архангельск
./run.sh "Архангельск" 64.5473 40.5602

# Петрозаводск
./run.sh "Петрозаводск" 61.7849 34.3469
```

### Зарубежные города
```bash
# Рейкьявик (Исландия) - отличное место для северных сияний
./run.sh "Рейкьявик" 64.1466 -21.9426

# Тромсё (Норвегия) - столица северных сияний
./run.sh "Тромсё" 69.6492 18.9553

# Фэрбанкс (Аляска) - золотой треугольник северных сияний
./run.sh "Фэрбанкс" 64.8378 -147.7164
```

## 🎯 Примеры использования для фотографов

### Планирование съемки на завтра
```bash
# Проверяем условия для съемки в Москве
./run.sh "Москва"

# Если условия хорошие, планируем локации
# Если есть северные сияния, едем за город
```

### Съемка северных сияний
```bash
# Проверяем условия в Мурманске
./run.sh "Мурманск" 68.9792 33.0925

# Если вероятность высокая, готовим оборудование:
# - Широкоугольный объектив
# - Штатив
# - Теплая одежда
# - Запасные батареи
```

### Городская фотография
```bash
# Проверяем золотой час в Санкт-Петербурге
./run.sh "Санкт-Петербург" 59.9311 30.3609

# Планируем съемку на набережных и мостах
```

## 🔧 Продвинутое использование

### Автоматический запуск по расписанию
```bash
# Добавьте в crontab для ежедневной проверки в 6:00
0 6 * * * cd /path/to/my_dashboard && ./run.sh "Москва" > /tmp/weather_dashboard.log 2>&1
```

### Сравнение условий в разных городах
```bash
#!/bin/bash
# Создайте скрипт compare_cities.sh
cities=(
    "Москва:55.7558:37.6176"
    "Санкт-Петербург:59.9311:30.3609"
    "Мурманск:68.9792:33.0925"
)

for city_info in "${cities[@]}"; do
    IFS=':' read -r city lat lon <<< "$city_info"
    echo "=== $city ==="
    ./run.sh "$city" "$lat" "$lon"
    echo ""
done
```

### Интеграция с уведомлениями
```bash
#!/bin/bash
# Создайте скрипт check_and_notify.sh
./run.sh "Москва" | grep -q "ОТЛИЧНО"

if [ $? -eq 0 ]; then
    # Отправляем уведомление
    notify-send "Фотография" "Отличные условия для съемки!"
    # Или отправляем в Telegram
    # curl -s "https://api.telegram.org/bot<BOT_TOKEN>/sendMessage" \
    #     -d "chat_id=<CHAT_ID>" \
    #     -d "text=Отличные условия для съемки!"
fi
```

## 🐛 Устранение неполадок

### Ошибка "Cargo не найден"
```bash
# Установите Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Ошибка "Permission denied"
```bash
# Сделайте скрипт исполняемым
chmod +x run.sh
```

### Ошибка API ключа
```bash
# Проверьте, что API ключ корректный
echo $OPENWEATHER_API_KEY

# Или установите новый
export OPENWEATHER_API_KEY="your_new_api_key"
```

## 📊 Интерпретация результатов

### Оценка погоды
- **8-10 баллов**: Отличные условия для съемки
- **6-7 баллов**: Хорошие условия
- **4-5 баллов**: Умеренные условия
- **0-3 балла**: Неблагоприятные условия

### Вероятность северных сияний
- **70-100%**: Высокая вероятность, готовьтесь к съемке
- **40-69%**: Умеренная вероятность, стоит попробовать
- **0-39%**: Низкая вероятность, лучше не тратить время

### Золотой час
- **Утренний**: 1 час до и после восхода
- **Вечерний**: 1 час до и после заката
- **Синий час**: 30 минут до восхода и после заката

---

**Удачной съемки! 📸** 