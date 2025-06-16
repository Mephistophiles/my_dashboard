// use regex::Regex;
use std::env;
use std::fs;
use std::process::Command;

// NOTE: Этот тест может обращаться к сети, если не выставлен DEMO режим или demo_key
#[test]
fn test_readme_output_matches_demo() {
    if std::env::var("GITHUB_ACTIONS").unwrap_or_default() == "true" {
        eprintln!("Skipped on CI");
        return;
    }
    // Устанавливаем DEMO режим
    env::set_var("DEMO_MODE", "true");
    env::set_var("OPENWEATHER_API_KEY", "demo_key");
    env::set_var("CITY", "Moscow");
    env::set_var("LATITUDE", "55.7558");
    env::set_var("LONGITUDE", "37.6176");

    // Запускаем main и захватываем вывод
    let output = Command::new("cargo")
        .args(["run", "--bin", "my_dashboard"])
        .env("DEMO_MODE", "true")
        .env("OPENWEATHER_API_KEY", "demo_key")
        .env("CITY", "Moscow")
        .env("LATITUDE", "55.7558")
        .env("LONGITUDE", "37.6176")
        .env("RUST_LOG", "error")
        .output()
        .expect("Не удалось запустить main");

    assert!(
        output.status.success(),
        "main завершился с ошибкой: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual_output = String::from_utf8(output.stdout)
        .expect("Не удалось декодировать вывод")
        .trim()
        .to_string();

    // Читаем README.md
    let readme_content = fs::read_to_string("README.md").expect("Не удалось прочитать README.md");

    // Ищем секцию с примером вывода в README.md
    // Предполагаем, что пример находится между ``` и ```
    let demo_section = extract_demo_section_from_readme(&readme_content);

    if let Some(expected_output) = demo_section {
        if actual_output != expected_output {
            println!("=== РАЗЛИЧИЯ В ВЫВОДЕ ===");
            println!("ОЖИДАЕМЫЙ (из README.md):");
            println!("{}", expected_output);
            println!("\nФАКТИЧЕСКИЙ (из main):");
            println!("{}", actual_output);
        }

        assert_eq!(
            actual_output, expected_output,
            "Вывод main не соответствует примеру в README.md"
        );
    } else {
        panic!("Не найдена секция с примером вывода в README.md. Добавьте блок кода с ```");
    }
}

fn extract_demo_section_from_readme(content: &str) -> Option<String> {
    let begin = "<!-- dashboard-demo-begin -->";
    let end = "<!-- dashboard-demo-end -->";
    let start = content.find(begin)? + begin.len();
    let rest = &content[start..];
    let end_idx = rest.find(end)?;
    let mut snippet = rest[..end_idx].trim();
    // Убираем обрамляющие тройные кавычки, если есть
    if snippet.starts_with("```") {
        snippet = snippet.trim_start_matches("```").trim_start();
    }
    if snippet.ends_with("```") {
        snippet = snippet.trim_end_matches("```").trim_end();
    }
    Some(snippet.to_string())
}
