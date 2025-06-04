use anyhow::Result;
use clap::Parser;
use dhd::{commands, config::Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "dhd", version, author, about = "Declarative Home Deployments")]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long, help = "Launch the TUI interface")]
    tui: bool,

    #[arg(long, help = "Launch the GUI interface")]
    gui: bool,

    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Parser)]
enum Command {
    Plan(PlanArgs),
    Apply(ApplyArgs),
    Tui,
}

#[derive(Parser)]
struct PlanArgs {
    #[arg(long)]
    modules: Option<Vec<String>>,

    #[arg(short = 'p', long)]
    modules_path: Option<std::path::PathBuf>,
}

#[derive(Parser)]
struct ApplyArgs {
    #[arg(long, default_value = "4")]
    max_concurrent: usize,

    #[arg(long)]
    modules: Option<Vec<String>>,

    #[arg(short = 'p', long)]
    modules_path: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
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

    // Handle TUI flag
    if cli.tui {
        return Ok(commands::tui::execute()?);
    }

    // Handle GUI flag
    if cli.gui {
        // Check if dist directory exists
        if !std::path::Path::new("dist").exists() {
            eprintln!(
                "Error: Frontend not built. Please run 'cargo build --features desktop' first."
            );
            std::process::exit(1);
        }

        // Launch the Tauri app
        return launch_gui();
    }

    let config = Config::load().unwrap_or_default();

    match cli.cmd {
        Some(Command::Plan(args)) => {
            let modules_path = args.modules_path.or(config.modules_path.map(Into::into));
            commands::plan::execute_and_display(args.modules, modules_path)?;
        }
        Some(Command::Apply(args)) => {
            let modules_path = args.modules_path.or(config.modules_path.map(Into::into));
            let max_concurrent = config.max_concurrent.unwrap_or(args.max_concurrent);
            commands::apply::execute(args.modules, modules_path, max_concurrent)?;
        }
        Some(Command::Tui) => {
            commands::tui::execute()?;
        }
        None => {
            eprintln!("No command specified. Use --help for usage information.");
            std::process::exit(1);
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
