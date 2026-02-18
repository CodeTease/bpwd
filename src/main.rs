use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::collections::HashMap;
use thiserror::Error;
use tinyjson::JsonValue;

/// Custom error types for bpwd
#[derive(Error, Debug)]
enum BwdError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Clipboard Error: {0}")]
    Clipboard(String),
    #[error("Invalid path: '{0}'")]
    InvalidPath(String),
    #[error("Root not found")]
    RootNotFound,
    #[error("JSON Error: {0}")]
    Json(String),
}

struct Config {
    target: Option<String>,
    copy: bool,
    short: bool,
    json: bool,
    root: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("[bwd error] {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), BwdError> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Check for help/version flags, but respect the -- separator.
    let flags_end = args.iter().position(|arg| arg == "--").unwrap_or(args.len());
    let flags_slice = &args[..flags_end];

    if flags_slice.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }

    if flags_slice.iter().any(|arg| arg == "-v" || arg == "--version") {
        println!("{} v{}", "bwd", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let config = parse_config(&args);

    let cwd = env::current_dir().map_err(BwdError::Io)?;

    let final_path = if let Some(t) = &config.target {
        let path = cwd.join(t);
        if !path.exists() {
            return Err(BwdError::InvalidPath(t.to_string()));
        }
        clean_windows_path(fs::canonicalize(path).map_err(BwdError::Io)?)
    } else {
        cwd
    };

    let absolute_str = final_path.to_string_lossy().to_string();

    // Determine home directory for shortening
    let home_dir = get_home_dir();

    // JSON Output Priority
    if config.json {
        let short_str = shorten_path(&final_path, home_dir.as_deref());
        
        let root_val = if let Some(root) = find_root(&final_path) {
             let relative = final_path.strip_prefix(&root).unwrap_or(Path::new(""));
             let s = if relative.as_os_str().is_empty() {
                 ".".to_string()
             } else {
                 relative.to_string_lossy().to_string()
             };
             JsonValue::String(s)
        } else {
            JsonValue::Null
        };
        
        let mut map = HashMap::new();
        map.insert("path".to_string(), JsonValue::String(absolute_str));
        map.insert("short".to_string(), JsonValue::String(short_str));
        map.insert("root".to_string(), root_val);
        
        let json_obj = JsonValue::Object(map);
        let json_str = json_obj.stringify().map_err(|e| BwdError::Json(format!("{:?}", e)))?;
        println!("{}", json_str);
        return Ok(());
    }

    // Short Output Priority
    if config.short {
        let short_str = shorten_path(&final_path, home_dir.as_deref());
        println!("{}", short_str);
        if config.copy {
            cli_clipboard::set_contents(short_str).map_err(|e| BwdError::Clipboard(e.to_string()))?;
        }
        return Ok(());
    }

    // Default Output Priority
    // Note: Previously logic handled -r here. If user passed -r but NOT -j or -s, 
    // should we still output relative path?
    // The prompt says "Default: In đường dẫn tuyệt đối".
    // But if explicit -r is passed, it's not "Default". 
    // I will preserve -r behavior if explicitly requested, otherwise default to absolute.
    let output_str = if config.root {
         if let Some(root) = find_root(&final_path) {
             let relative = final_path.strip_prefix(&root).unwrap_or(Path::new(""));
             if relative.as_os_str().is_empty() {
                 ".".to_string()
             } else {
                 relative.to_string_lossy().to_string()
             }
        } else {
            return Err(BwdError::RootNotFound);
        }
    } else {
        absolute_str
    };

    println!("{}", output_str);

    if config.copy {
        cli_clipboard::set_contents(output_str).map_err(|e| BwdError::Clipboard(e.to_string()))?;
    }

    Ok(())
}

fn parse_config(args: &[String]) -> Config {
    let mut target = None;
    let mut copy = false;
    let mut short = false;
    let mut json = false;
    let mut root = false;
    let mut parsing_flags = true;

    for arg in args {
        if parsing_flags && arg == "--" {
            parsing_flags = false;
            continue;
        }

        if parsing_flags && arg.starts_with('-') {
            match arg.as_str() {
                "-c" | "--copy" => copy = true,
                "-s" | "--short" => short = true,
                "-j" | "--json" => json = true,
                "-r" | "--root" => root = true,
                _ => {} // Ignore unknown flags
            }
            continue;
        }

        // If it's not a flag (or we stopped parsing flags), it's the target
        if target.is_none() {
            target = Some(arg.clone());
        }
    }
    Config { target, copy, short, json, root }
}

fn get_home_dir() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
        .or_else(|| env::var("USERPROFILE").ok().map(PathBuf::from))
}

fn shorten_path(path: &Path, home: Option<&Path>) -> String {
    if let Some(h) = home {
        if let Ok(stripped) = path.strip_prefix(h) {
             let replacement = if stripped.as_os_str().is_empty() {
                 PathBuf::from("$HOME")
             } else {
                 PathBuf::from("$HOME").join(stripped)
             };
             return replacement.to_string_lossy().to_string();
        }
    }
    path.to_string_lossy().to_string()
}

fn find_root(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.join(".git").exists() || current.join(".bwd-root").exists() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(p) => current = p,
            None => return None,
        }
    }
}

/// Strip the UNC prefix (\\?\$ which is common on Windows when using canonicalize()
fn clean_windows_path(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with(r"\\?\") {
        PathBuf::from(&path_str[4..])
    } else {
        path
    }
}

fn print_help() {
    println!("bwd - Better Working Directory");
    println!("\nUsage:");
    println!("  bwd [target] [-c] [-s] [-j] [-r]");
    println!("\nFlags:");
    println!("  -c, --copy     Copy to clipboard");
    println!("  -s, --short    Shorten path (replace home with $HOME)");
    println!("  -j, --json     Output JSON (path, short, root)");
    println!("  -r, --root     Print path relative to project root (.git or .bwd-root)");
    println!("  -h, --help     Show this help");
    println!("  -v, --version  Show version");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_config_defaults() {
        let args: Vec<String> = vec![];
        let config = parse_config(&args);
        assert_eq!(config.target, None);
        assert_eq!(config.copy, false);
        assert_eq!(config.short, false);
        assert_eq!(config.json, false);
        assert_eq!(config.root, false);
    }

    #[test]
    fn test_parse_config_short_flag() {
        let args: Vec<String> = vec!["-s".to_string()];
        let config = parse_config(&args);
        assert!(config.short);
        assert!(!config.json);
    }

    #[test]
    fn test_parse_config_json_flag() {
        let args: Vec<String> = vec!["--json".to_string()];
        let config = parse_config(&args);
        assert!(config.json);
        assert!(!config.short);
    }

    #[test]
    fn test_parse_config_all_flags() {
        let args: Vec<String> = vec!["-c".to_string(), "-s".to_string(), "-j".to_string()];
        let config = parse_config(&args);
        assert!(config.copy);
        assert!(config.short);
        assert!(config.json);
    }

    #[test]
    fn test_shorten_path_match() {
        // Construct paths in a platform-agnostic way for testing logic
        let home = PathBuf::from("/home/user");
        let path = home.join("docs/project");
        let short = shorten_path(&path, Some(&home));
        
        // Expected: $HOME/docs/project
        // Note: join uses OS separator. On unix it's /, on windows \
        // The shorten_path implementation uses PathBuf::from("$HOME").join(...)
        // So it should match OS separator.
        let expected = PathBuf::from("$HOME").join("docs/project").to_string_lossy().to_string();
        assert_eq!(short, expected);
    }

    #[test]
    fn test_shorten_path_exact_match() {
        let home = PathBuf::from("/home/user");
        let short = shorten_path(&home, Some(&home));
        assert_eq!(short, "$HOME");
    }

    #[test]
    fn test_shorten_path_no_match() {
        let home = PathBuf::from("/home/user");
        let path = PathBuf::from("/var/log");
        let short = shorten_path(&path, Some(&home));
        assert_eq!(short, path.to_string_lossy().to_string());
    }

    #[test]
    fn test_shorten_path_no_home() {
        let path = PathBuf::from("/home/user/docs");
        let short = shorten_path(&path, None);
        assert_eq!(short, path.to_string_lossy().to_string());
    }

    #[test]
    fn test_clean_windows_path_no_prefix() {
        let p = PathBuf::from("/usr/bin");
        assert_eq!(clean_windows_path(p.clone()), p);
    }

    #[test]
    fn test_clean_windows_path_with_prefix() {
        let p = PathBuf::from(r"\\?\C:\Windows");
        let expected = PathBuf::from(r"C:\Windows");
        assert_eq!(clean_windows_path(p), expected);
    }

    #[test]
    fn test_parse_config_target_only() {
        let args: Vec<String> = vec!["some/path".to_string()];
        let config = parse_config(&args);
        assert_eq!(config.target, Some("some/path".to_string()));
    }

    #[test]
    fn test_parse_config_ignore_unknown_flags_as_target() {
        // Unknown flags are ignored, so target remains None unless it's positional
        // In the loop: if not parsing flags, or not starting with -, it's target.
        // If it starts with - and is unknown, it's ignored.
        let args: Vec<String> = vec!["-x".to_string()];
        let config = parse_config(&args);
        assert_eq!(config.target, None);
        // But if we have -x followed by path?
        let args2: Vec<String> = vec!["-x".to_string(), "path".to_string()];
        let config2 = parse_config(&args2);
        assert_eq!(config2.target, Some("path".to_string()));
    }

    #[test]
    fn test_parse_config_dash_separator() {
        let args: Vec<String> = vec!["--".to_string(), "-file".to_string()];
        let config = parse_config(&args);
        assert_eq!(config.target, Some("-file".to_string()));
        assert!(!config.copy);
    }

    #[test]
    fn test_parse_config_dash_separator_with_flags() {
        let args: Vec<String> = vec!["-c".to_string(), "--".to_string(), "-file".to_string()];
        let config = parse_config(&args);
        assert_eq!(config.target, Some("-file".to_string()));
        assert!(config.copy);
    }

    #[test]
    fn test_parse_config_flags_after_separator_are_target() {
        let args: Vec<String> = vec!["--".to_string(), "-c".to_string()];
        let config = parse_config(&args);
        assert_eq!(config.target, Some("-c".to_string()));
        assert!(!config.copy);
    }

    #[test]
    fn test_parse_config_root_flag() {
        let args: Vec<String> = vec!["-r".to_string()];
        let config = parse_config(&args);
        assert!(config.root);
    }

    #[test]
    fn test_find_root_git() {
        let temp_dir = std::env::temp_dir();
        let test_root = temp_dir.join(format!("bpwd_test_git_{}", process::id()));
        if test_root.exists() {
            let _ = fs::remove_dir_all(&test_root);
        }
        fs::create_dir_all(&test_root).unwrap();
        fs::create_dir(test_root.join(".git")).unwrap();
        
        let child = test_root.join("subdir");
        fs::create_dir(&child).unwrap();

        assert_eq!(find_root(&child), Some(test_root.clone()));
        assert_eq!(find_root(&test_root), Some(test_root.clone()));

        // Cleanup
        let _ = fs::remove_dir_all(&test_root);
    }

    #[test]
    fn test_find_root_bwd() {
        let temp_dir = std::env::temp_dir();
        let test_root = temp_dir.join(format!("bpwd_test_bwd_{}", process::id()));
        if test_root.exists() {
            let _ = fs::remove_dir_all(&test_root);
        }
        fs::create_dir_all(&test_root).unwrap();
        fs::create_dir(test_root.join(".bwd-root")).unwrap();
        
        let child = test_root.join("subdir/deep");
        fs::create_dir_all(&child).unwrap();

        assert_eq!(find_root(&child), Some(test_root.clone()));

        // Cleanup
        let _ = fs::remove_dir_all(&test_root);
    }
}