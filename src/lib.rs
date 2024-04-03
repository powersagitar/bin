#![feature(absolute_path)]

pub fn add(
    source_binaries: &[std::path::PathBuf],
    destination_directory: &std::path::Path,
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
                return Err(format!("Failed to get binary name for {:?}", source_binary));
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
            return Err(format!(
                "Failed to check if {:?} already exists on $PATH",
                binary_name
            ));
        }

        let source_binary_absolute = {
            let source_binary_absolute_result = std::path::absolute(source_binary);

            if let Ok(source_binary_absolute) = source_binary_absolute_result {
                source_binary_absolute
            } else {
                return Err(format!(
                    "Failed to get absolute path of {:?}",
                    source_binary
                ));
            }
        };

        let target_symlink = destination_directory.join(binary_name);

        if std::os::unix::fs::symlink(&source_binary_absolute, &target_symlink).is_err() {
            return Err(format!(
                "Failed to create symlink {:?} -> {:?}",
                target_symlink, source_binary_absolute
            ));
        }
    }

    Ok(())
}

pub fn prune(directory: &std::path::Path) -> Result<(), String> {
    let read_dir = {
        let read_dir_result = std::fs::read_dir(directory);

        if let Ok(read_dir) = read_dir_result {
            read_dir
        } else {
            return Err(format!("Failed to read directory {:?}", directory));
        }
    };

    for dir_entry_result in read_dir {
        if let Ok(dir_entry) = dir_entry_result {
            if !dir_entry.path().exists() && std::fs::remove_file(dir_entry.path()).is_err() {
                return Err(format!(
                    "Failed to remove file/symlink {:?}",
                    dir_entry.path()
                ));
            }
        } else {
            return Err(format!(
                "Failed to unwrap directory entry {:?}",
                dir_entry_result
            ));
        }
    }

    Ok(())
}
