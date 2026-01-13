use std::{collections::HashMap, env, fs, io::Write, path::PathBuf, process};

use clap::{Parser, Subcommand};
use colored::{Colorize, control::set_override};
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

const EXIT_CD: i32 = 42;

const INDEX_WIDTH_BASE: usize = 2;

#[derive(Parser)]
#[command(name = "hop")]
#[command(about = "Quickly hop between saved directories")]
struct Cli {
    #[arg(value_name = "SHORTCUT")]
    shortcut: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(alias = "a")]
    Add { name: String },

    #[command(alias = "ls")]
    List,

    #[command(alias = "rm")]
    Remove { name: String },

    #[command(alias = "f")]
    Fuzzy,

    Init {
        #[arg(short, long, value_name = "SHELL")]
        shell: Option<String>,

        #[arg(long)]
        install: bool,
    },
}

fn main() {
    set_override(true);

    let cli = Cli::parse();

    match &cli.command {
        Some(command) => handle_command(command),
        None => try_handle_shortcut(&cli.shortcut),
    }
}

fn handle_command(command: &Command) {
    match command {
        Command::Add { name } => handle_add(name),
        Command::List => handle_list(),
        Command::Remove { name } => handle_remove(name),
        Command::Fuzzy => handle_fuzzy(),
        Command::Init { shell, install } => handle_init(shell, *install),
    }
}

fn try_handle_shortcut(shortcut: &Option<String>) {
    match shortcut {
        Some(shortcut) => {
            let config = get_config_path().expect("Could not find config directory");
            let shortcuts = load_shortcuts(&config).unwrap_or_else(|e| {
                eprintln!("Error loading shortcuts: {}", e);
                process::exit(1);
            });

            if let Some(path) = shortcuts.get(shortcut) {
                println!("{}", expand_path(path));
                process::exit(EXIT_CD);
            } else {
                eprintln!("Shortcut `{}` not found", shortcut);
                process::exit(1);
            }
        }
        None => handle_list(),
    }
}

fn get_config_path() -> Option<PathBuf> {
    let mut path = dirs::config_dir()?;

    path.push("hop");
    path.push("paths.csv");

    Some(path)
}

fn normalize_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path.starts_with(home_str.as_ref()) {
            return path.replacen(home_str.as_ref(), "~", 1);
        }
    }
    path.to_string()
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    } else if path == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home.to_string_lossy().to_string();
    }
    path.to_string()
}

#[derive(Serialize, Deserialize)]
struct Shortcut {
    name: String,
    path: String,
}

fn load_shortcuts(path: &PathBuf) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let mut reader = ReaderBuilder::new().has_headers(false).from_path(path)?;
    let mut shortcuts = HashMap::new();
    for result in reader.deserialize() {
        let shortcut: Shortcut = result?;
        shortcuts.insert(shortcut.name, shortcut.path);
    }
    Ok(shortcuts)
}

fn save_shortcuts(
    path: &PathBuf,
    shortcuts: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut writer = WriterBuilder::new().has_headers(false).from_path(path)?;
    for (name, path) in shortcuts {
        let shortcut = Shortcut {
            name: name.clone(),
            path: path.clone(),
        };
        writer.serialize(&shortcut)?;
    }
    writer.flush()?;
    Ok(())
}

fn handle_list() {
    let config = get_config_path().expect("Could not find config directory");
    let shortcuts = load_shortcuts(&config).unwrap_or_else(|e| {
        eprintln!("Error loading shortcuts: {}", e);
        process::exit(1);
    });

    if shortcuts.is_empty() {
        println!("No shortcuts yet. Use `hop add <name>` to add one.");
        return;
    }

    let max_shortcut_len = shortcuts.keys().map(|s| s.len()).max().unwrap_or(0);

    for (name, path) in &shortcuts {
        let padded_name = format!("{:<max_shortcut_len$}", name);
        println!("{} -> {}", padded_name.bold().green(), path.blue());
    }
}

fn handle_add(name: &str) {
    if name.is_empty() {
        eprintln!("Shortcut name cannot be empty");
        return;
    }

    let config = get_config_path().expect("Could not find config directory");
    let mut shortcuts = load_shortcuts(&config).unwrap_or_else(|e| {
        eprintln!("Error loading shortcuts: {}", e);
        process::exit(1);
    });

    if shortcuts.contains_key(name) {
        println!("Shortcut `{name}` already exists");
        return;
    }

    let current_dir = env::current_dir().expect("Could not get current directory");
    let expanded_path = current_dir.to_string_lossy().to_string();
    if !fs::metadata(&expanded_path).is_ok_and(|m| m.is_dir()) {
        eprintln!("Current directory is not valid");
        return;
    }

    let path = normalize_path(&expanded_path);

    shortcuts.insert(name.to_string(), path.clone());

    println!("Added: {} -> {}", name.bold().green(), path.blue());

    if let Err(e) = save_shortcuts(&config, &shortcuts) {
        eprintln!("Could not save shortcuts: {}", e);
        process::exit(1);
    }
}

fn handle_remove(name: &str) {
    let config = get_config_path().expect("Could not find config directory");
    let mut shortcuts = load_shortcuts(&config).unwrap_or_else(|e| {
        eprintln!("Error loading shortcuts: {}", e);
        process::exit(1);
    });

    if !shortcuts.contains_key(name) {
        println!("Shortcut `{name}` does not exist");
        return;
    }

    shortcuts.remove(name);

    if let Err(e) = save_shortcuts(&config, &shortcuts) {
        eprintln!("Could not save shortcuts: {}", e);
        process::exit(1);
    }
}

fn handle_fuzzy() {
    let config = get_config_path().expect("Could not find config directory");
    let shortcuts = load_shortcuts(&config).unwrap_or_else(|e| {
        eprintln!("Error loading shortcuts: {}", e);
        process::exit(1);
    });

    if shortcuts.is_empty() {
        eprintln!("No shortcuts yet.");
        return;
    }

    let shortcuts_vec: Vec<_> = shortcuts.iter().collect();

    let index_width = shortcuts.len().to_string().len() + INDEX_WIDTH_BASE;

    let input = shortcuts_vec
        .iter()
        .enumerate()
        .map(|(i, (name, path))| {
            format!(
                "{:<iw$}{} -> {}",
                format!("{}.", i + 1),
                name,
                path,
                iw = index_width,
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let mut child = process::Command::new("fzf")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| {
            eprintln!("Failed to start `fzf`. Check if it is installed.");
            process::exit(1);
        });

    if let Some(ref mut stdin) = child.stdin
        && let Err(e) = stdin.write_all(input.as_bytes())
    {
        eprintln!("Failed to write to `fzf`: {}", e);
        let _ = child.wait();
        return;
    }

    let output = child.wait_with_output().unwrap_or_else(|e| {
        eprintln!("Failed to wait for `fzf`: {}", e);
        process::exit(1);
    });

    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let Some((_, path_str)) = selection.split_once(" -> ") else {
            eprintln!("Failed to parse selection");
            return;
        };

        println!("{}", expand_path(path_str));
        process::exit(EXIT_CD);
    }
}

fn handle_init(shell: &Option<String>, install: bool) {
    let shell_name = shell.as_deref().unwrap_or("bash");
    let script: &'static str = match shell_name {
        "bash" | "zsh" => include_str!("../contrib/hop.bash"),
        "fish" => include_str!("../contrib/hop.fish"),
        "powershell" | "pwsh" | "ps1" => include_str!("../contrib/hop.ps1"),
        other => {
            eprintln!("Unsupported shell: {}", other);
            eprintln!("Supported shells: bash, zsh, fish, powershell");
            return;
        }
    };

    if install {
        if let Some(mut path) = dirs::config_dir() {
            path.push("hop");
            let filename = match shell_name {
                "fish" => "hop.fish",
                "powershell" | "pwsh" | "ps1" => "hop.ps1",
                _ => "hop.sh",
            };
            path.push(filename);

            if let Some(parent) = path.parent()
                && let Err(e) = fs::create_dir_all(parent)
            {
                eprintln!("Failed to create directory {}: {}", parent.display(), e);
                return;
            }

            match fs::write(&path, script) {
                Ok(_) => println!("Wrote initialization script to {}", path.to_string_lossy()),
                Err(e) => eprintln!("Failed to write init script: {}", e),
            }
        } else {
            eprintln!(
                "Could not determine config directory. Remove --install or specify a valid config directory."
            );
        }
    } else {
        print!("{}", script);
    }
}
