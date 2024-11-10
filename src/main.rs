use clap::Parser;
use std::path::PathBuf;

use conflink::ConflinkConfig;

#[derive(clap::Parser)]
struct CliConfig {
    /// Force overwrite symlinks.
    #[arg(short, long)]
    force: bool,

    /// Configuration file to use.
    #[arg(short, long)]
    config_file: PathBuf,
}

fn main() {
    let cli_config = CliConfig::parse();

    if !cli_config.config_file.exists() {
        eprintln!(
            "Config file '{}' not found.",
            cli_config.config_file.display()
        );
        std::process::exit(1);
    }

    let config = std::fs::read_to_string(cli_config.config_file).expect("No such file.");

    let configs = match toml::from_str::<ConflinkConfig>(&config) {
        Ok(mut config) => config.prepare_links(),
        Err(err) => {
            eprintln!("{err:#?}");
            std::process::exit(1);
        }
    };

    for link_config in configs {
        if !link_config.apply {
            continue;
        }

        println!(
            "Linking: {} -> {}",
            link_config.link_path.display(),
            link_config.link_to.display(),
        );

        if link_config.link_path.exists() {
            if cli_config.force {
                let Ok(_) = std::fs::remove_file(&link_config.link_path) else {
                    eprintln!(
                        "\tLink '{}' already exists and could not be removed.",
                        link_config.link_path.display()
                    );
                    std::process::exit(1);
                };
            } else {
                eprintln!(
                    "\tLink '{}' already exists, skipping...",
                    link_config.link_path.display()
                );
                continue;
            }
        }

        if let Err(err) = rustix::fs::symlink(link_config.link_to, link_config.link_path) {
            eprintln!("\tERROR: {err}");
        }
    }
}
