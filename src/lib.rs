pub fn add(
    source_binaries: &[std::path::PathBuf],
    destination_dir: &std::path::Path,
) -> Result<(), &'static str> {
    for source_binary in source_binaries {
        let binary_name = {
            let binary_name_option = source_binary.file_name();

            if let Some(binary_name) = binary_name_option {
                binary_name
            } else {
                return Err("Failed to get binary name");
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
            return Err("Failed to check if binary exists");
        }

        if std::os::unix::fs::symlink(source_binary, destination_dir.join(binary_name)).is_err() {
            return Err("Failed to symlink binary");
        }
    }

    Ok(())
}
