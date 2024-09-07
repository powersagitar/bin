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

pub fn remove(binaries: &[std::ffi::OsString]) -> Result<(), String> {
    for bin in binaries {
        let bin_install_path = {
            let bin_install_path_result = bin_path(bin);

            match bin_install_path_result {
                Ok(Some(bin_install_path)) => bin_install_path,
                Ok(None) => {
                    return Err(format!(
                        "Binary {:?} does not exist on $PATH, nothing to remove",
                        bin
                    ))
                }
                Err(err) => {
                    return Err(format!(
                        "Failed to check if {:?} already exists on $PATH: {}",
                        bin, err
                    ))
                }
            }
        };

        if let Err(err) = std::fs::remove_file(&bin_install_path) {
            return Err(format!(
                "Failed to remove file/symlink {:?}: {:?}",
                bin_install_path, err
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir_base() -> std::path::PathBuf {
        "/tmp/libbin-tests".into()
    }

    fn test_dir() -> std::path::PathBuf {
        static mut TEST_COUNTER: u32 = 0;
        test_dir_base().join(format!("test-{}", unsafe {
            TEST_COUNTER += 1;
            TEST_COUNTER
        }))
    }

    fn test_bin_install_path(test_dir: &std::path::Path) -> std::path::PathBuf {
        test_dir.join("bin")
    }

    fn initialize_test_dir(test_dir: &std::path::Path) {
        if test_dir.exists() {
            std::fs::remove_dir_all(&test_dir).unwrap();
        }

        std::fs::create_dir_all(&test_dir).unwrap();

        let bin_install_path = test_bin_install_path(test_dir);

        std::fs::create_dir(bin_install_path).unwrap();
    }

    #[test]
    fn test_bin_path_valid() {
        let result = bin_path(std::ffi::OsStr::new("which"));

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_bin_path_invalid() {
        let result = bin_path(std::ffi::OsStr::new("nonexistent"));

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_add_successful() {
        let test_dir = test_dir();

        initialize_test_dir(&test_dir);

        let source_binary = std::path::PathBuf::from(test_dir.join("test-bin"));

        std::fs::write(&source_binary, b"#!/bin/sh\necho test").unwrap();

        add(&[source_binary], &test_bin_install_path(&test_dir)).unwrap();

        assert!(test_bin_install_path(&test_dir).join("test-bin").exists());
    }

    #[test]
    fn test_add_invalid_bin() {
        let test_dir = test_dir();

        initialize_test_dir(&test_dir);

        let source_binary = std::path::PathBuf::from("/nonexistent");

        let result = add(&[source_binary], &test_bin_install_path(&test_dir));

        assert!(result.is_err());
    }

    #[test]
    fn test_add_existing_bin() {
        let test_dir = test_dir();

        initialize_test_dir(&test_dir);

        let source_binary = test_dir.join("which");

        std::fs::write(&source_binary, "").unwrap();

        add(&[source_binary], &test_bin_install_path(&test_dir)).unwrap();

        assert!(!test_bin_install_path(&test_dir).join("which").exists());
    }

    #[test]
    fn test_prune_preserve_valid_symlinks() {
        let test_dir = test_dir();

        initialize_test_dir(&test_dir);

        let bin_install_path = test_bin_install_path(&test_dir);

        let source_binary = std::path::PathBuf::from("/bin/ls");

        std::os::unix::fs::symlink(&source_binary, bin_install_path.join("ls")).unwrap();

        prune(&bin_install_path).unwrap();

        assert!(bin_install_path.join("ls").exists());
    }

    #[test]
    fn test_prune_remove_void_symlinks() {
        let test_dir = test_dir();

        initialize_test_dir(&test_dir);

        let bin_install_path = test_bin_install_path(&test_dir);

        let source_binary = std::path::PathBuf::from("/nonexistent");

        std::os::unix::fs::symlink(&source_binary, bin_install_path.join("nonexistent")).unwrap();

        prune(&bin_install_path).unwrap();

        assert!(!bin_install_path.join("nonexistent").exists());
    }

    #[test]
    fn test_remove_successful() {
        let test_dir = test_dir();
        initialize_test_dir(&test_dir);

        let test_bin_install_path = test_bin_install_path(&test_dir);

        let path = std::env::var_os("PATH").unwrap();
        let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
        paths.push(test_bin_install_path.clone());
        let new_path = std::env::join_paths(paths).unwrap();
        std::env::set_var("PATH", &new_path);

        let test_bin = test_bin_install_path.join("libbin-test-bin");
        std::fs::write(&test_bin, "").unwrap();
        std::fs::set_permissions(
            &test_bin,
            std::os::unix::fs::PermissionsExt::from_mode(0b111_101_101),
        )
        .unwrap();

        remove(&["libbin-test-bin".into()]).unwrap();

        assert!(!test_bin.exists());
    }

    #[test]
    fn test_remove_nonexistent() {
        let result = remove(&["nonexistent".into()]);

        assert!(result.is_err());
    }
}
