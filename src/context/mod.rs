use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub project_types: Vec<String>,
    pub os_info: String,
    pub shell: String,
}

impl ProjectContext {
    pub fn detect() -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();
        let project_types = detect_project_types(&cwd);
        let os_info = detect_os_info();
        let shell = detect_shell();

        Self {
            project_types,
            os_info,
            shell,
        }
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.project_types.is_empty() {
            parts.push(format!("Project types: {}", self.project_types.join(", ")));
        }
        parts.push(format!("OS: {}", self.os_info));
        parts.push(format!("Shell: {}", self.shell));

        parts.join("\n")
    }
}

pub fn detect_project_types(dir: &Path) -> Vec<String> {
    let mut types = Vec::new();

    let checks: &[(&str, &str)] = &[
        ("package.json", "Node.js"),
        ("Cargo.toml", "Rust"),
        ("requirements.txt", "Python"),
        ("pyproject.toml", "Python"),
        ("go.mod", "Go"),
        ("Dockerfile", "Docker"),
        ("docker-compose.yml", "Docker Compose"),
        ("docker-compose.yaml", "Docker Compose"),
        ("Makefile", "Make"),
    ];

    for (file, label) in checks {
        if dir.join(file).exists() {
            types.push(label.to_string());
        }
    }

    // Check for GitHub Actions
    if dir.join(".github").join("workflows").exists() {
        types.push("GitHub Actions".to_string());
    }

    // Check for Terraform files
    if has_files_with_extension(dir, "tf") {
        types.push("Terraform".to_string());
    }

    // Check for Kubernetes manifests
    if has_k8s_manifests(dir) {
        types.push("Kubernetes".to_string());
    }

    // Deduplicate
    types.dedup();
    types
}

fn has_files_with_extension(dir: &Path, ext: &str) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .any(|e| e.path().extension().is_some_and(|x| x == ext))
        })
        .unwrap_or(false)
}

fn has_k8s_manifests(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                let path = e.path();
                if path.extension().is_some_and(|x| x == "yaml" || x == "yml") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        return content.contains("apiVersion:");
                    }
                }
                false
            })
        })
        .unwrap_or(false)
}

fn detect_os_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!("{os} ({arch})")
}

fn detect_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_nodejs_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Node.js".to_string()));
    }

    #[test]
    fn test_detect_rust_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Rust".to_string()));
    }

    #[test]
    fn test_detect_python_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "flask").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Python".to_string()));
    }

    #[test]
    fn test_detect_go_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("go.mod"), "module example").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Go".to_string()));
    }

    #[test]
    fn test_detect_docker_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Dockerfile"), "FROM ubuntu").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Docker".to_string()));
    }

    #[test]
    fn test_detect_terraform_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("main.tf"), "resource {}").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Terraform".to_string()));
    }

    #[test]
    fn test_detect_k8s_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("deployment.yaml"),
            "apiVersion: apps/v1\nkind: Deployment",
        )
        .unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"Kubernetes".to_string()));
    }

    #[test]
    fn test_detect_github_actions() {
        let dir = TempDir::new().unwrap();
        let workflows = dir.path().join(".github").join("workflows");
        std::fs::create_dir_all(&workflows).unwrap();
        std::fs::write(workflows.join("ci.yml"), "on: push").unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.contains(&"GitHub Actions".to_string()));
    }

    #[test]
    fn test_empty_directory() {
        let dir = TempDir::new().unwrap();
        let types = detect_project_types(dir.path());
        assert!(types.is_empty());
    }

    #[test]
    fn test_os_info() {
        let info = detect_os_info();
        assert!(!info.is_empty());
    }
}
