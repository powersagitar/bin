pub fn add(source_binaries: &[std::path::PathBuf], destination: &std::path::Path) {
    for source_binary in source_binaries {
        std::os::unix::fs::symlink(
            source_binary,
            destination.join(source_binary.file_name().expect("Failed to get file name")),
        )
        .expect("Failed to symlink binary");
    }
}
