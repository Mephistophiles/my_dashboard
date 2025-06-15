# Justfile для проекта my_dashboard

# Инструкция:
# Для всех проверок (coverage, tests) используйте just с нужной целью, например:
#   just test
#   just coverage
#   just coverage-strict
# Это гарантирует использование правильных путей и настроек (например, --target-dir).

# Запуск приложения
run:
    cargo run

# Запуск тестов
test:
    cargo test

# Проверка покрытия кода (HTML отчет)
coverage:
    cargo tarpaulin --out Html --target-dir target/tarpaulin

# Проверка покрытия с порогом 55%
coverage-strict:
    cargo tarpaulin --out Html --fail-under 55 --target-dir target/tarpaulin

# Проверка покрытия с подробным отчетом
coverage-detail:
    cargo tarpaulin --out Html --out Xml --target-dir target/tarpaulin

# Форматирование кода
fmt:
    cargo fmt --all

# Проверка стиля и предупреждений (строгий режим)
clippy:
    cargo clippy -- -D warnings

# Проверка стиля и предупреждений (с исправлениями)
clippy-fix:
    cargo clippy --fix -- -D warnings

# Быстрая сборка
build:
    cargo build

# Сборка в release режиме
build-release:
    cargo build --release

# Генерация документации
doc:
    cargo doc --open

# Очистка сборки
clean:
    cargo clean

# Проверка всех тестов и качества кода
check: fmt clippy test coverage-strict

# Полная проверка перед коммитом
pre-commit: check build


# Release-процедура с параметром версии
# Использование: just release 0.5.0
release version: pre-commit build-release
    git add .
    git commit -m "release: v{{version}}"
    git tag v{{version}} -m "release: v{{version}}"