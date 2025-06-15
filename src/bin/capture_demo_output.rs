use std::env;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .env("RUST_LOG", "error") // Убираем логи для чистого вывода
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let demo_output = stdout.trim();

        println!("=== ЗАХВАЧЕННЫЙ ВЫВОД MAIN В DEMO РЕЖИМЕ ===\n");
        println!("```");
        println!("{}", demo_output);
        println!("```");
        // Автоматически обновляем README.md
        update_readme_with_demo_output(demo_output)?;
        println!("README.md автоматически обновлен!");
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        eprintln!("Ошибка запуска: {}", stderr);
        return Err("Ошибка запуска main".into());
    }

    Ok(())
}

fn update_readme_with_demo_output(demo_output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let readme_path = "README.md";
    let readme_content = fs::read_to_string(readme_path)?;

    let begin_marker = "<!-- dashboard-demo-begin -->";
    let end_marker = "<!-- dashboard-demo-end -->";

    let begin_pos = readme_content
        .find(begin_marker)
        .ok_or("Не найден маркер <!-- dashboard-demo-begin --> в README.md")?;
    let end_pos = readme_content
        .find(end_marker)
        .ok_or("Не найден маркер <!-- dashboard-demo-end --> в README.md")?;

    // Создаем новое содержимое README.md
    let before_demo = &readme_content[..begin_pos + begin_marker.len()];
    let after_demo = &readme_content[end_pos..];

    let new_readme_content = format!("{}\n```\n{}\n```\n{}", before_demo, demo_output, after_demo);

    // Создаем backup перед изменением
    fs::write("README.md.backup", &readme_content)?;

    // Записываем обновленный README.md
    fs::write(readme_path, new_readme_content)?;

    println!("Создан backup: README.md.backup");

    Ok(())
}
