use clap::{Parser, Subcommand};
use serde_json::{Map, Value};

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
    /// Show context information available to conditions
    #[command(alias = "ctx")]
    Context {
        #[command(subcommand)]
        context_command: ContextCommands,
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
        /// Enable verbose output including condition evaluations
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    Types,
}

#[derive(Subcommand)]
enum ContextCommands {
    /// List all available context properties and their current values
    #[command(alias = "ls")]
    List,
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

fn list_contexts() -> Result<(), String> {
    use dhd::system_info::get_system_info;
    
    let info = get_system_info();
    
    // Serialize SystemInfo to JSON to iterate over fields dynamically
    let json_value = serde_json::to_value(&info)
        .map_err(|e| format!("Failed to serialize system info: {}", e))?;
    
    println!("Available context properties:");
    println!("{:<30} {:<40} {}", "Property Path", "Current Value", "Type");
    println!("{}", "-".repeat(85));
    
    if let Value::Object(map) = json_value {
        print_properties(&map, "", None);
    }
    
    println!("\nThese properties can be used in module conditions with the property() function.");
    println!("Example: property(\"os.family\").equals(\"debian\")");
    
    Ok(())
}

fn print_properties(map: &Map<String, Value>, prefix: &str, _category: Option<&str>) {
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| k.as_str());
    
    for (key, value) in entries {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };
        
        match value {
            Value::Object(nested_map) => {
                // Print category header
                let category_name = key.to_uppercase();
                println!("\n{}", category_name);
                
                // Recursively print nested properties
                print_properties(nested_map, &path, Some(&category_name));
            }
            Value::String(s) => {
                println!("{:<30} {:<40} {}", path, s, "string");
            }
            Value::Bool(b) => {
                println!("{:<30} {:<40} {}", path, b, "boolean");
            }
            Value::Number(n) => {
                println!("{:<30} {:<40} {}", path, n, "number");
            }
            _ => {
                // Skip null or array values
            }
        }
    }
}

fn list_modules() -> Result<(), String> {
    use dhd::{discover_modules, load_modules};
    use std::env;

    let current_dir =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Discover modules with progress
    print!("● Discovering TypeScript modules...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let discovered =
        discover_modules(&current_dir).map_err(|e| format!("Failed to discover modules: {}", e))?;
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
        println!(
            "Found {} TypeScript module(s):",
            loaded_modules.len() + failed_modules.len()
        );
        for module in &loaded_modules {
            if let Some(relative) = module.source.relative_path(&current_dir) {
                let description = module
                    .definition
                    .description
                    .as_ref()
                    .map(|d| format!(" - {}", d))
                    .unwrap_or_default();
                let tags = if module.definition.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", module.definition.tags.join(", "))
                };
                println!(
                    "  - {} ({}){}{}",
                    module.definition.name,
                    relative.display(),
                    tags,
                    description
                );
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

fn apply_modules(
    dry_run: bool,
    module_filters: Vec<String>,
    tag_filters: Vec<String>,
    all_tags: bool,
    verbose: bool,
) -> Result<(), String> {
    use dhd::{ExecutionEngine, discover_modules, load_modules, dependency_resolver::resolve_dependencies};
    use std::env;

    let current_dir =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Discover all modules with progress
    print!("● Discovering TypeScript modules...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let discovered =
        discover_modules(&current_dir).map_err(|e| format!("Failed to discover modules: {}", e))?;
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
        return Err(format!(
            "No modules could be loaded ({} failed)",
            failed_count
        ));
    }

    if failed_count > 0 {
        println!("\n● Warning: {} modules failed to load", failed_count);
    }

    // Filter modules by their actual names and tags
    let filtered_modules: Vec<_> = loaded_modules
        .iter()
        .filter(|module| {
            // Check module name filter
            let name_match =
                module_filters.is_empty() || module_filters.contains(&module.definition.name);

            // Check tag filter
            let tag_match = if tag_filters.is_empty() {
                true
            } else if all_tags {
                // Module must have ALL specified tags
                tag_filters
                    .iter()
                    .all(|tag| module.definition.tags.contains(tag))
            } else {
                // Module must have AT LEAST ONE of the specified tags
                tag_filters
                    .iter()
                    .any(|tag| module.definition.tags.contains(tag))
            };

            name_match && tag_match
        })
        .cloned()
        .collect();

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
            println!(
                "ℹ️  No modules matched the specified filters: {}",
                filters.join("; ")
            );
        }
        return Ok(());
    }

    // Include dependencies of selected modules
    let mut modules_with_deps = filtered_modules.clone();
    let mut added_deps = true;
    
    while added_deps {
        added_deps = false;
        let current_names: Vec<String> = modules_with_deps.iter()
            .map(|m| m.definition.name.clone())
            .collect();
        
        for module in modules_with_deps.clone() {
            for dep in &module.definition.dependencies {
                if !current_names.contains(dep) {
                    if let Some(dep_module) = loaded_modules.iter().find(|m| &m.definition.name == dep) {
                        modules_with_deps.push(dep_module.clone());
                        added_deps = true;
                    } else {
                        return Err(format!(
                            "Module '{}' depends on '{}' which was not found",
                            module.definition.name, dep
                        ));
                    }
                }
            }
        }
    }
    
    // Resolve dependencies to get correct execution order
    let resolved_modules = resolve_dependencies(modules_with_deps)
        .map_err(|e| format!("Failed to resolve dependencies: {}", e))?;

    // Show selected modules
    println!(
        "\n● Selected modules for {}:",
        if dry_run { "dry run" } else { "execution" }
    );
    for (idx, module) in resolved_modules.iter().enumerate() {
        let action_count = module.definition.actions.len();
        let description = module
            .definition
            .description
            .as_ref()
            .map(|d| format!(" - {}", d))
            .unwrap_or_default();
        let tags = if module.definition.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", module.definition.tags.join(", "))
        };
        let prefix = if idx == resolved_modules.len() - 1 {
            "  ⎿"
        } else {
            "  ├"
        };
        println!(
            "{} {} ({} action{}){}{}",
            prefix,
            module.definition.name,
            action_count,
            if action_count == 1 { "" } else { "s" },
            tags,
            description
        );
    }

    // Execute modules
    println!(); // Add spacing before execution

    // Use default concurrency (number of CPUs) for execution
    let concurrency = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4); // Default to 4 if we can't determine CPU count
    let engine = ExecutionEngine::new(concurrency, dry_run, verbose);

    // Execute the modules
    match engine.execute(resolved_modules) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Execution failed: {}", e)),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { generate_command } => match generate_command {
            GenerateCommands::Types => {
                if let Err(e) = generate_types() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::Context { context_command } => match context_command {
            ContextCommands::List => {
                if let Err(e) = list_contexts() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::List => {
            if let Err(e) = list_modules() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Apply {
            dry_run,
            module,
            tag,
            all_tags,
            verbose,
        } => {
            if let Err(e) = apply_modules(dry_run, module, tag, all_tags, verbose) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
