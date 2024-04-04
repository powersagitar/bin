#![feature(absolute_path)]

pub fn bin_path(bin: &std::ffi::OsStr) -> Result<Option<std::path::PathBuf>, String> {
    match std::process::Command::new("which").arg(bin).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(Some(
                    // Omit trailing line feed
                    match std::str::from_utf8(&output.stdout[..output.stdout.len() - 1]) {
                        Ok(path) => path.into(),
                        Err(err) => return Err(format!("Failed to parse which output: {}", err)),
                    },
                ))
            } else {
                Ok(None)
            }
        }
        Err(err) => Err(format!(
            "Failed to check if {:?} already exists on $PATH: {}",
            bin, err
        )),
    }
}

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

        match bin_path(binary_name) {
            Ok(Some(path)) => {
                eprintln!("{:?} already exists as {:?}, skipping", binary_name, path);
                continue;
            }
            Ok(None) => {}
            Err(err) => return Err(err),
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

        match read_dir_result {
            Ok(read_dir) => read_dir,
            Err(err) => return Err(format!("Failed to read directory {:?}: {}", directory, err)),
        }
    };

    for dir_entry_result in read_dir {
        match dir_entry_result {
            Ok(dir_entry) => {
                if !dir_entry.path().exists() {
                    if let Err(err) = std::fs::remove_file(dir_entry.path()) {
                        return Err(format!(
                            "Failed to remove file/symlink {:?}: {}",
                            dir_entry.path(),
                            err
                        ));
                    }
                }
            }
            Err(err) => return Err(format!("Failed to unwrap directory entry: {}", err)),
        }
    }

    Ok(())
}
