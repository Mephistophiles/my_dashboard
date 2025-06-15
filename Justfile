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
fmt-check:
    cargo fmt --check

# Проверка стиля и предупреждений (строгий режим)
clippy:
    cargo clippy --all-targets -- -D warnings

# Проверка стиля и предупреждений (с исправлениями)
clippy-fix:
    cargo clippy --fix --all-targets -- -D warnings

# Проверка линтинга
lint:
    cargo check --all-targets

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
check: fmt-check lint clippy test

# Полная проверка перед коммитом
pre-commit: check build


# Release-процедура с параметром версии
# Использование: just release 0.5.0
#
# ПЕРЕД РЕЛИЗОМ ОБЯЗАТЕЛЬНО:
# 1. Обновите CHANGELOG.md - переместите изменения из [Unreleased] в новую версию
# 2. Убедитесь, что все тесты проходят: just test
# 3. Проверьте качество кода: just clippy
# 4. Убедитесь, что покрытие кода соответствует требованиям: just coverage-strict
#
release version: pre-commit build-release
    git add .
    git commit -m "release: v{{version}}"
    git tag v{{version}} -m "release: v{{version}}"

# Захват вывода main в DEMO режиме и автоматическое обновление README.md
capture-demo:
    cargo run --bin capture_demo_output

# Проверка соответствия вывода main с README.md
test-readme-snapshot:
    cargo test test_readme_output_matches_demo --test readme_snapshot_test
