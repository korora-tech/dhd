use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dhd")]
#[command(about = "Module Discovery and Execution")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[command(subcommand)]
        generate_command: GenerateCommands,
    },
    /// List all discovered TypeScript modules
    List,
    /// Apply (execute) discovered modules
    Apply {
        /// Run in dry-run mode to preview what would be executed
        #[arg(long)]
        dry_run: bool,
        /// Filter to specific modules by name (can be used multiple times)
        #[arg(long, value_name = "MODULE")]
        module: Vec<String>,
        /// Filter to modules with specific tags (can be used multiple times)
        #[arg(long, value_name = "TAG")]
        tag: Vec<String>,
        /// Filter to modules with ALL specified tags
        #[arg(long)]
        all_tags: bool,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    Types,
}

fn generate_types() -> Result<(), String> {
    use std::fs;

    // Get all TypeScript definitions from our macros
    let typescript_definitions = dhd::typescript::generate_typescript_definitions();

    // Write to types.d.ts
    fs::write("types.d.ts", typescript_definitions)
        .map_err(|e| format!("Failed to write types.d.ts: {}", e))?;

    // Generate tsconfig.json with global types
    let tsconfig_content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "node",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "allowSyntheticDefaultImports": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true
  },
  "include": [
    "**/*.ts",
    "types.d.ts"
  ],
  "exclude": [
    "node_modules",
    "dist",
    "build",
    ".git"
  ]
}"#;

    fs::write("tsconfig.json", tsconfig_content)
        .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;

    println!("TypeScript definitions generated in types.d.ts");
    println!("TypeScript configuration generated in tsconfig.json");
    Ok(())
}

fn list_modules() -> Result<(), String> {
    use dhd::{discover_modules, load_modules};
    use std::env;

    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Discover modules with progress
    print!("● Discovering TypeScript modules...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let discovered = discover_modules(&current_dir)
        .map_err(|e| format!("Failed to discover modules: {}", e))?;
    println!(" found {}", discovered.len());

    if discovered.is_empty() {
        println!("No TypeScript modules found");
        return Ok(());
    }

    // Load modules to get their actual names with progress
    println!("● Loading module definitions...");

    let load_results = load_modules(discovered.clone());
    let mut loaded_modules = Vec::new();
    let mut failed_modules = Vec::new();

    for (i, result) in load_results.into_iter().enumerate() {
        match result {
            Ok(loaded) => loaded_modules.push(loaded),
            Err(_) => failed_modules.push(&discovered[i]),
        }
    }

    // Show loaded modules with their actual names
    if !loaded_modules.is_empty() {
        println!("Found {} TypeScript module(s):", loaded_modules.len() + failed_modules.len());
        for module in &loaded_modules {
            if let Some(relative) = module.source.relative_path(&current_dir) {
                let description = module.definition.description.as_ref()
                    .map(|d| format!(" - {}", d))
                    .unwrap_or_default();
                let tags = if module.definition.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", module.definition.tags.join(", "))
                };
                println!("  - {} ({}){}{}", module.definition.name, relative.display(), tags, description);
            }
        }
    }

    // Show failed modules
    if !failed_modules.is_empty() {
        if !loaded_modules.is_empty() {
            println!();
        }
        println!("Failed to load {} module(s):", failed_modules.len());
        for module in &failed_modules {
            if let Some(relative) = module.relative_path(&current_dir) {
                println!("  - {} ({})", module.name, relative.display());
            }
        }
    }

    Ok(())
}

fn apply_modules(dry_run: bool, module_filters: Vec<String>, tag_filters: Vec<String>, all_tags: bool) -> Result<(), String> {
    use dhd::{discover_modules, load_modules, ExecutionEngine, ExecutionMode};
    use std::env;

    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Discover all modules with progress
    print!("● Discovering TypeScript modules...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let discovered = discover_modules(&current_dir)
        .map_err(|e| format!("Failed to discover modules: {}", e))?;
    println!(" found {}", discovered.len());

    if discovered.is_empty() {
        println!("No TypeScript modules found in current directory");
        return Ok(());
    }

    // Load all modules first to get their actual names with progress
    println!("● Loading and validating modules...");

    let load_results = load_modules(discovered.clone());
    let mut loaded_modules = Vec::new();
    let mut failed_count = 0;

    for (i, result) in load_results.into_iter().enumerate() {
        match result {
            Ok(loaded) => loaded_modules.push(loaded),
            Err(e) => {
                eprintln!("  ⎿ ✗ Failed to load module {}: {}", discovered[i].name, e);
                failed_count += 1;
            }
        }
    }

    if loaded_modules.is_empty() {
        return Err(format!("No modules could be loaded ({} failed)", failed_count));
    }

    if failed_count > 0 {
        println!("\n● Warning: {} modules failed to load", failed_count);
    }

    // Filter modules by their actual names and tags
    let filtered_modules = loaded_modules.into_iter()
        .filter(|module| {
            // Check module name filter
            let name_match = module_filters.is_empty() ||
                module_filters.contains(&module.definition.name);

            // Check tag filter
            let tag_match = if tag_filters.is_empty() {
                true
            } else if all_tags {
                // Module must have ALL specified tags
                tag_filters.iter().all(|tag| module.definition.tags.contains(tag))
            } else {
                // Module must have AT LEAST ONE of the specified tags
                tag_filters.iter().any(|tag| module.definition.tags.contains(tag))
            };

            name_match && tag_match
        })
        .collect::<Vec<_>>();

    if filtered_modules.is_empty() {
        if module_filters.is_empty() && tag_filters.is_empty() {
            println!("ℹ️  No modules to execute");
        } else {
            let mut filters = Vec::new();
            if !module_filters.is_empty() {
                filters.push(format!("modules: {}", module_filters.join(", ")));
            }
            if !tag_filters.is_empty() {
                let tag_op = if all_tags { "all of" } else { "any of" };
                filters.push(format!("tags ({}): {}", tag_op, tag_filters.join(", ")));
            }
            println!("ℹ️  No modules matched the specified filters: {}", filters.join("; "));
        }
        return Ok(());
    }

    // Show selected modules
    println!("\n● Selected modules for {}:", if dry_run { "dry run" } else { "execution" });
    for (idx, module) in filtered_modules.iter().enumerate() {
        let action_count = module.definition.actions.len();
        let description = module.definition.description.as_ref()
            .map(|d| format!(" - {}", d))
            .unwrap_or_default();
        let tags = if module.definition.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", module.definition.tags.join(", "))
        };
        let prefix = if idx == filtered_modules.len() - 1 { "  ⎿" } else { "  ├" };
        println!("{} {} ({} action{}){}{}",
            prefix,
            module.definition.name,
            action_count,
            if action_count == 1 { "" } else { "s" },
            tags,
            description
        );
    }

    // Execute modules
    let mode = if dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Execute
    };

    println!(); // Add spacing before execution
    let engine = ExecutionEngine::new(mode);

    // Execute with dependency resolution
    match engine.execute_modules_with_dependencies(filtered_modules) {
        Ok(result) => {
            // The execution engine already prints a detailed summary,
            // so we just need to check for errors and exit codes
            if result.failed > 0 {
                return Err(format!("Execution failed: {} error(s) occurred", result.failed));
            }
            Ok(())
        }
        Err(dependency_error) => {
            Err(format!("Dependency resolution failed: {}", dependency_error))
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { generate_command } => {
            match generate_command {
                GenerateCommands::Types => {
                    if let Err(e) = generate_types() {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::List => {
            if let Err(e) = list_modules() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Apply { dry_run, module, tag, all_tags } => {
            if let Err(e) = apply_modules(dry_run, module, tag, all_tags) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
