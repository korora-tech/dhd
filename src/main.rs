use anyhow::Result;
use clap::Parser;
use dhd::{commands, config::Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "dhd", version, author, about = "Declarative Home Deployments")]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// Generate a plan for the specified modules
    Plan {
        /// Module names to plan (e.g., dhd plan podman neovim)
        modules: Vec<String>,

        #[arg(short = 'p', long)]
        modules_path: Option<std::path::PathBuf>,

        /// Filter by tags (e.g., --tags dev,tools)
        #[arg(short = 't', long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
    /// Apply the specified modules
    Apply {
        /// Module names to apply (e.g., dhd apply podman neovim)
        modules: Vec<String>,

        #[arg(long, default_value = "4")]
        max_concurrent: usize,

        #[arg(short = 'p', long)]
        modules_path: Option<std::path::PathBuf>,

        /// Filter by tags (e.g., --tags dev,tools)
        #[arg(short = 't', long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
    /// List available modules
    List {
        #[arg(short = 'p', long)]
        modules_path: Option<std::path::PathBuf>,

        /// Filter by tags (e.g., --tags dev,tools)
        #[arg(short = 't', long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
    /// Launch the TUI interface
    Tui,
    /// Launch the GUI interface  
    Gui,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging only for non-TUI modes
    let is_tui = matches!(cli.cmd, Command::Tui);

    if !is_tui {
        let log_level = match cli.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        };

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("dhd={}", log_level).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    } else {
        // For TUI mode, configure logging to a file or disable it entirely
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/dhd-tui.log")
            .ok();

        if let Some(file) = log_file {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "dhd=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer().with_writer(std::sync::Arc::new(file)))
                .init();
        }
    }

    let config = Config::load().unwrap_or_default();

    match cli.cmd {
        Command::Plan {
            modules,
            modules_path,
            tags,
        } => {
            let modules_path = modules_path.or(config.modules_path.map(Into::into));
            let modules_opt = if modules.is_empty() && tags.is_none() {
                None
            } else if modules.is_empty() {
                // If only tags provided, this will be handled in the plan command
                None
            } else {
                Some(modules)
            };
            commands::plan::execute_and_display(modules_opt, modules_path, tags)?;
        }
        Command::Apply {
            modules,
            modules_path,
            max_concurrent,
            tags,
        } => {
            let modules_path = modules_path.or(config.modules_path.map(Into::into));
            let max_concurrent = config.max_concurrent.unwrap_or(max_concurrent);
            let modules_opt = if modules.is_empty() && tags.is_none() {
                None
            } else if modules.is_empty() {
                // If only tags provided, this will be handled in the apply command
                None
            } else {
                Some(modules)
            };
            commands::apply::execute(modules_opt, modules_path, max_concurrent, tags)?;
        }
        Command::List { modules_path, tags } => {
            let modules_path = modules_path.or(config.modules_path.map(Into::into));
            commands::list::execute(modules_path, tags)?;
        }
        Command::Tui => {
            commands::tui::execute()?;
        }
        Command::Gui => {
            // Check if dist directory exists
            if !std::path::Path::new("dist").exists() {
                eprintln!(
                    "Error: Frontend not built. Please run 'cargo build --features desktop' first."
                );
                std::process::exit(1);
            }

            // Launch the Tauri app
            launch_gui()?;
        }
    }

    Ok(())
}

fn launch_gui() -> Result<()> {
    use dhd::gui::AppState;
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            // Setup app state with current working directory
            let modules_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let state = AppState { modules_path };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dhd::gui::commands::list_modules,
            dhd::gui::commands::generate_plan,
            dhd::gui::commands::apply_modules
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
