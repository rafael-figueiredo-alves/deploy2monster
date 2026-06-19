use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use crate::logger;
use crate::projects::Project;

pub fn run(project: &Project) -> Result<(), String> {
    let output_dir = build_output_dir(project)?;

    clean_output_dir(&output_dir)?;

    run_dotnet_publish(project, &output_dir)?;

    Ok(())
}

fn build_output_dir(project: &Project) -> Result<String, String> {
    let base = Path::new(&project.publish_folder);

    if !base.exists() {
        return Err(format!(
            "Pasta de publicação não encontrada: {}",
            project.publish_folder
        ));
    }

    let output = base.join(&project.name);
    Ok(output.to_string_lossy().to_string())
}

fn clean_output_dir(output_dir: &str) -> Result<(), String> {
    let path = Path::new(output_dir);

    if path.exists() {
        logger::info(&format!("Limpando pasta: {}", output_dir));
        fs::remove_dir_all(path)
            .map_err(|e| format!("Erro ao limpar pasta de publicação: {}", e))?;
    }

    fs::create_dir_all(path)
        .map_err(|e| format!("Erro ao criar pasta de publicação: {}", e))?;

    Ok(())
}

fn run_dotnet_publish(project: &Project, output_dir: &str) -> Result<(), String> {
    logger::info("Executando dotnet publish...");
    logger::info(&format!("  Projeto : {}", project.project_file));
    logger::info(&format!("  Destino : {}", output_dir));

    let status = Command::new("dotnet")
        .args([
            "publish",
            &project.project_file,
            "-c", "Release",
            "--self-contained",
            "-r", "win-x86",
            "-o", output_dir,
        ])
        .stdout(Stdio::inherit()) // exibe output em tempo real
        .stderr(Stdio::inherit()) // exibe erros em tempo real
        .status()
        .map_err(|e| format!("Erro ao executar dotnet: {}", e))?;

    if !status.success() {
        return Err(format!(
            "dotnet publish falhou com código: {}",
            status.code().unwrap_or(-1)
        ));
    }

    logger::info("Publicação concluída.");
    Ok(())
}