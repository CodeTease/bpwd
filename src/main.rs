use std::env;
use std::fmt;
use std::fs;
use std::path::{PathBuf};
use std::process;

/// Custom error types for bpwd
enum BwdError {
    Io(std::io::Error),
    Clipboard(String),
    InvalidPath(String),
}

impl fmt::Display for BwdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BwdError::Io(err) => write!(f, "IO Error: {}", err),
            BwdError::Clipboard(err) => write!(f, "Clipboard Error: {}", err),
            BwdError::InvalidPath(path) => write!(f, "Invalid path: '{}'", path),
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("[bwd error] {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), BwdError> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }

    let (target, copy_flag, slash_flag) = parse_config(&args);

    let cwd = env::current_dir().map_err(BwdError::Io)?;

    let final_path = if let Some(t) = target {
        let path = cwd.join(&t);
        if !path.exists() {
            return Err(BwdError::InvalidPath(t.to_string()));
        }
        clean_windows_path(fs::canonicalize(path).map_err(BwdError::Io)?)
    } else {
        cwd
    };

    let mut path_str = final_path.to_string_lossy().to_string();

    // Apply slash normalization if -s flag is present
    if slash_flag {
        path_str = path_str.replace('\\', "/");
    }

    // Standard output
    println!("{}", path_str);

    // Clipboard action
    if copy_flag {
        cli_clipboard::set_contents(path_str).map_err(|e| BwdError::Clipboard(e.to_string()))?;
    }

    Ok(())
}

fn parse_config(args: &[String]) -> (Option<String>, bool, bool) {
    let copy_flag = args.iter().any(|arg| arg == "-c");
    let slash_flag = args.iter().any(|arg| arg == "-s");
    let target = args.iter().find(|arg| !arg.starts_with('-')).cloned();
    (target, copy_flag, slash_flag)
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
    println!("  bwd [target] [-c] [-s]");
    println!("\nFlags:");
    println!("  -c             Copy to clipboard");
    println!("  -s             Use forward slashes (/) instead of backslashes (\\$");
    println!("  -h, --help     Show this help");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_config_no_args() {
        let args: Vec<String> = vec![];
        let (target, copy, slash) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, false);
    }

    #[test]
    fn test_parse_config_target_only() {
        let args: Vec<String> = vec!["some/path".to_string()];
        let (target, copy, slash) = parse_config(&args);
        assert_eq!(target, Some("some/path".to_string()));
        assert_eq!(copy, false);
        assert_eq!(slash, false);
    }

    #[test]
    fn test_parse_config_copy_flag() {
        let args: Vec<String> = vec!["-c".to_string()];
        let (target, copy, slash) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, true);
        assert_eq!(slash, false);
    }

    #[test]
    fn test_parse_config_slash_flag() {
        let args: Vec<String> = vec!["-s".to_string()];
        let (target, copy, slash) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, true);
    }

    #[test]
    fn test_parse_config_all() {
        let args: Vec<String> = vec!["-c".to_string(), "target".to_string(), "-s".to_string()];
        let (target, copy, slash) = parse_config(&args);
        assert_eq!(target, Some("target".to_string()));
        assert_eq!(copy, true);
        assert_eq!(slash, true);
    }
    
    #[test]
    fn test_parse_config_ignore_unknown_flags_as_target() {
        let args: Vec<String> = vec!["-x".to_string()];
        let (target, copy, slash) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, false);
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
}
