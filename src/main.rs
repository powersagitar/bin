const DEFAULT_BIN_INSTALL_PATH: &str = "/usr/local/bin";

fn main() {
    let args = clap::command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .next_line_help(true)
        .subcommand(
            clap::Command::new("add")
                .about("Symlink binaries to destination")
                .alias("install")
                .arg(
                    clap::Arg::new("BINARIES")
                        .help("Paths to binaries to symlink")
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .action(clap::ArgAction::Append)
                        .required(true),
                )
                .arg(
                    clap::Arg::new("DESTINATION")
                        .help("Destination directory to symlink binaries")
                        .short('d')
                        .long("destination")
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .default_value(DEFAULT_BIN_INSTALL_PATH),
                ),
        )
        .subcommand(
            clap::Command::new("remove")
                .about("Remove binaries from $PATH")
                .alias("rm")
                .alias("uninstall")
                .alias("delete")
                .arg(
                    clap::Arg::new("BINARIES")
                        .help("Paths to binaries to remove")
                        .value_parser(clap::value_parser!(std::ffi::OsString))
                        .action(clap::ArgAction::Append)
                        .required(true),
                ),
        )
        .subcommand(
            clap::Command::new("prune")
                .about("Remove symlinks resolving to void")
                .arg(
                    clap::Arg::new("DIRECTORY")
                        .help("Target directory to prune")
                        .short('d')
                        .long("directory")
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .default_value(DEFAULT_BIN_INSTALL_PATH),
                ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("add", args)) => {
            let source_binaries: Vec<std::path::PathBuf> = args
                .get_many::<std::path::PathBuf>("BINARIES")
                .expect("Failed to parse source binaries")
                .map(|path_buf_ref| path_buf_ref.into())
                .collect();

            let destination: std::path::PathBuf = args
                .get_one::<std::path::PathBuf>("DESTINATION")
                .expect("Failed to parse destination")
                .into();

            if let Err(err) = bin::add(&source_binaries, &destination) {
                panic!("{}", err);
            };
        }

        Some(("remove", args)) => {
            let binaries: Vec<std::ffi::OsString> = args
                .get_many::<std::ffi::OsString>("BINARIES")
                .expect("Failed to parse binaries")
                .map(|os_string_ref| os_string_ref.into())
                .collect();

            if let Err(err) = bin::remove(&binaries) {
                panic!("{}", err);
            };
        }

        Some(("prune", args)) => {
            let destination: std::path::PathBuf = args
                .get_one::<std::path::PathBuf>("DIRECTORY")
                .expect("Failed to parse target directory")
                .into();

            if let Err(err) = bin::prune(&destination) {
                panic!("{}", err);
            };
        }
        _ => unreachable!("Unimplemented subcommand"),
    }
}
