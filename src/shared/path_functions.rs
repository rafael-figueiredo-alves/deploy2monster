use std::path::PathBuf;

pub fn resolve_project_path(name: &str) -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter caminho do executável: {}", e))?;

    let projects_dir = exe_path.parent()
        .ok_or_else(|| "Não foi possível determinar o diretório do executável".to_string())?
        .join("projects");

    Ok(projects_dir.join(format!("{}.d2mproj", name)))
}