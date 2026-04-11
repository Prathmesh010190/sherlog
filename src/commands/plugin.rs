use anyhow::Result;
use colored::Colorize;

use crate::cli::PluginAction;
use crate::output::{self, OutputMode};
use crate::plugins::PluginManager;

pub async fn run(action: &PluginAction, mode: OutputMode) -> Result<()> {
    let manager = PluginManager::new()?;

    match action {
        PluginAction::Install { url } => {
            if mode == OutputMode::Terminal {
                output::print_info(&format!("Installing plugin from {url}..."));
            }

            let manifest = manager.install(url).await?;

            match mode {
                OutputMode::Json | OutputMode::Ci => {
                    let json = serde_json::to_string_pretty(&manifest)?;
                    println!("{json}");
                }
                OutputMode::Terminal => {
                    output::print_success(&format!(
                        "Installed plugin: {} v{}",
                        manifest.name, manifest.version
                    ));
                    if !manifest.description.is_empty() {
                        println!("  {}", manifest.description.dimmed());
                    }
                }
            }
        }

        PluginAction::List => {
            let manifests = manager.list_manifests()?;

            match mode {
                OutputMode::Json | OutputMode::Ci => {
                    let json = serde_json::to_string_pretty(&manifests)?;
                    println!("{json}");
                }
                OutputMode::Terminal => {
                    if manifests.is_empty() {
                        output::print_info("No plugins installed.");
                        println!(
                            "  Install one with: {}",
                            "sherlog plugin install <url>".cyan()
                        );
                    } else {
                        println!("{}", "Installed Plugins".cyan().bold());
                        println!("{}", "\u{2500}".repeat(40).dimmed());
                        for m in &manifests {
                            let caps: Vec<_> =
                                m.capabilities.iter().map(|c| format!("{c:?}")).collect();
                            println!(
                                "  {} {} {}",
                                m.name.white().bold(),
                                format!("v{}", m.version).dimmed(),
                                format!("[{}]", caps.join(", ")).dimmed()
                            );
                            if !m.description.is_empty() {
                                println!("    {}", m.description.dimmed());
                            }
                        }
                    }
                    println!();
                    println!(
                        "  {} {}",
                        "Plugins directory:".dimmed(),
                        manager.plugins_dir().display()
                    );
                }
            }
        }

        PluginAction::Remove { name } => {
            manager.remove(name)?;

            match mode {
                OutputMode::Json | OutputMode::Ci => {
                    println!(r#"{{"status": "ok", "removed": "{name}"}}"#);
                }
                OutputMode::Terminal => {
                    output::print_success(&format!("Removed plugin: {name}"));
                }
            }
        }
    }

    Ok(())
}
