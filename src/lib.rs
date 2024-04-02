#![feature(absolute_path)]
pub fn add(
    source_binaries: &[std::path::PathBuf],
    destination_dir: &std::path::Path,
) -> Result<(), String> {
    for source_binary in source_binaries {
        if !source_binary.exists() {
            return Err(format!("Source binary {:?} does not exist", source_binary));
        }

        let binary_name = {
            let binary_name_option = source_binary.file_name();

            if let Some(binary_name) = binary_name_option {
                binary_name
            } else {
                return Err("Failed to get binary name".into());
            }
        };

        if let Ok(output) = std::process::Command::new("which")
            .arg(binary_name)
            .output()
        {
            if !output.stdout.is_empty() {
                eprintln!(
                    "{:?} already exists as {:?}, skipping",
                    binary_name,
                    // Omit trailing line feed
                    String::from_utf8_lossy(&output.stdout[..output.stdout.len() - 1])
                );
                continue;
            }
        } else {
            return Err("Failed to check if binary exists".into());
        }

        let source_binary_absolute = {
            let source_binary_absolute_result = std::path::absolute(source_binary);

            if let Ok(source_binary_absolute) = source_binary_absolute_result {
                source_binary_absolute
            } else {
                return Err("Failed to get absolute source binary path".into());
            }
        };

        if std::os::unix::fs::symlink(source_binary_absolute, destination_dir.join(binary_name))
            .is_err()
        {
            return Err("Failed to symlink binary".into());
        }
    }

    Ok(())
}
