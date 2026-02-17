use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use thiserror::Error;

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

    let (target, copy_flag, slash_flag, root_flag) = parse_config(&args);

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

    let mut path_str = if root_flag {
        let root = find_root(&final_path).ok_or(BwdError::RootNotFound)?;
        let relative = final_path.strip_prefix(&root).unwrap_or(Path::new(""));
        if relative.as_os_str().is_empty() {
            ".".to_string()
        } else {
            relative.to_string_lossy().to_string()
        }
    } else {
        final_path.to_string_lossy().to_string()
    };

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

fn parse_config(args: &[String]) -> (Option<String>, bool, bool, bool) {
    let mut target = None;
    let mut copy_flag = false;
    let mut slash_flag = false;
    let mut root_flag = false;
    let mut parsing_flags = true;

    for arg in args {
        if parsing_flags && arg == "--" {
            parsing_flags = false;
            continue;
        }

        if parsing_flags && arg.starts_with('-') {
            match arg.as_str() {
                "-c" => copy_flag = true,
                "-s" => slash_flag = true,
                "-r" => root_flag = true,
                _ => {} // Ignore unknown flags
            }
            continue;
        }

        // If it's not a flag (or we stopped parsing flags), it's the target
        if target.is_none() {
            target = Some(arg.clone());
        }
    }
    (target, copy_flag, slash_flag, root_flag)
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
    println!("  bwd [target] [-c] [-s] [-r]");
    println!("\nFlags:");
    println!("  -c             Copy to clipboard");
    println!("  -s             Use forward slashes (/) instead of backslashes (\\$");
    println!("  -r             Print path relative to project root (.git or .bwd-root)");
    println!("  -h, --help     Show this help");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_config_no_args() {
        let args: Vec<String> = vec![];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, false);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_target_only() {
        let args: Vec<String> = vec!["some/path".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, Some("some/path".to_string()));
        assert_eq!(copy, false);
        assert_eq!(slash, false);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_copy_flag() {
        let args: Vec<String> = vec!["-c".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, true);
        assert_eq!(slash, false);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_slash_flag() {
        let args: Vec<String> = vec!["-s".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, true);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_all() {
        let args: Vec<String> = vec!["-c".to_string(), "target".to_string(), "-s".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, Some("target".to_string()));
        assert_eq!(copy, true);
        assert_eq!(slash, true);
        assert_eq!(root, false);
    }
    
    #[test]
    fn test_parse_config_ignore_unknown_flags_as_target() {
        let args: Vec<String> = vec!["-x".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, false);
        assert_eq!(root, false);
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
    fn test_parse_config_dash_separator() {
        let args: Vec<String> = vec!["--".to_string(), "-file".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, Some("-file".to_string()));
        assert_eq!(copy, false);
        assert_eq!(slash, false);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_dash_separator_with_flags() {
        let args: Vec<String> = vec!["-c".to_string(), "--".to_string(), "-file".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, Some("-file".to_string()));
        assert_eq!(copy, true);
        assert_eq!(slash, false);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_flags_after_separator_are_target() {
        let args: Vec<String> = vec!["--".to_string(), "-c".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, Some("-c".to_string()));
        assert_eq!(copy, false);
        assert_eq!(slash, false);
        assert_eq!(root, false);
    }

    #[test]
    fn test_parse_config_root_flag() {
        let args: Vec<String> = vec!["-r".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, None);
        assert_eq!(copy, false);
        assert_eq!(slash, false);
        assert_eq!(root, true);
    }

    #[test]
    fn test_parse_config_root_mixed() {
        let args: Vec<String> = vec!["-s".to_string(), "-r".to_string(), "foo".to_string()];
        let (target, copy, slash, root) = parse_config(&args);
        assert_eq!(target, Some("foo".to_string()));
        assert_eq!(copy, false);
        assert_eq!(slash, true);
        assert_eq!(root, true);
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