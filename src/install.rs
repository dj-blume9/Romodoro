use std::{env, fs, path::PathBuf};

pub fn ensure_global_install() -> bool {
    let exe_path = env::current_exe().expect("Failed to determine executable path.");

    #[cfg(target_family = "windows")]
    {
       use std::process::Command;  
        let home_dir =
            env::var("USERPROFILE").expect("Failed to get USERPROFILE environment variable");
        let user_bin_dir = PathBuf::from(format!(r"{}\{}", home_dir, ".local\\bin"));
        let global_exe_path = user_bin_dir.join("romo.exe");

        if !global_exe_path.exists() {
            println!("Installing Romo globally to user directory...");

            // Create the directory if it doesn't exist
            if !user_bin_dir.exists() {
                fs::create_dir_all(&user_bin_dir).expect("Failed to create user bin directory.");
            }

            // Copy the executable
            fs::copy(&exe_path, &global_exe_path)
                .expect("Failed to copy Romo to user bin directory.");

            // Add to PATH if necessary
            add_to_path_windows(&user_bin_dir);
            println!("Romo installed successfully! Restart your terminal to use 'romo' globally.");
            return false;
        }
        return true;
    }

    #[cfg(target_family = "unix")]
    {
        let local_bin_dir = PathBuf::from(env::var("HOME").unwrap()).join(".local/bin");
        let global_exe_path = local_bin_dir.join("romo");

        if !global_exe_path.exists() {
            println!("Installing Romo globally to ~/.local/bin...");

            // Create the directory if it doesn't exist
            if !local_bin_dir.exists() {
                fs::create_dir_all(&local_bin_dir)
                    .expect("Failed to create ~/.local/bin directory.");
            }

            // Copy the executable
            fs::copy(&exe_path, &global_exe_path).expect("Failed to copy Romo to ~/.local/bin.");

            // Add ~/.local/bin to PATH if necessary
            add_to_path_linux(&local_bin_dir);
            println!("Romo installed successfully! Restart your terminal to use 'romo' globally.");
            return false;
        }
        return true;
    }
}

#[cfg(target_family = "windows")]
fn add_to_path_windows(user_bin_dir: &PathBuf) {
    
    use std::process::Command;
    let path_to_add = user_bin_dir.to_str().unwrap();

    // Modify the User PATH variable using PowerShell
    Command::new("powershell")
        .args([
            "-Command",
            &format!(
                r#"$currentPath = [Environment]::GetEnvironmentVariable('Path', 'User');
                if ($currentPath -notlike '*{}*') {{
                    [Environment]::SetEnvironmentVariable('Path', ($currentPath + ';{}'), 'User')
                }}"#,
                path_to_add, path_to_add
            ),
        ])
        .status()
        .expect("Failed to add Romo to PATH on Windows.");
}

#[cfg(target_family = "unix")]
fn add_to_path_linux(local_bin_dir: &PathBuf) {
    let path_to_add = local_bin_dir.to_str().unwrap();

    let shell_rc = env::var("SHELL").unwrap_or_default();
    let rc_file = if shell_rc.contains("zsh") {
        ".zshrc"
    } else {
        ".bashrc"
    };

    let home_dir = env::var("HOME").unwrap();
    let rc_path = PathBuf::from(format!("{}/{}", home_dir, rc_file));

    // Add ~/.local/bin to PATH if not already present
    if let Ok(contents) = fs::read_to_string(&rc_path) {
        if !contents.contains(&format!("export PATH={}:$PATH", path_to_add)) {
            let line_to_add = format!("export PATH={}:$PATH", path_to_add);
            fs::write(&rc_path, format!("{}\n{}", contents, line_to_add))
                .expect("Failed to update PATH in shell configuration file.");
            println!(
                "Added ~/.local/bin to PATH in {}. Restart your terminal to use 'romo'.",
                rc_file
            );
        }
    } else {
        println!(
            "Could not open {}. Ensure ~/.local/bin is in your PATH.",
            rc_file
        );
    }
}
