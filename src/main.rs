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
                        .default_value("/usr/local/bin"),
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
        _ => unreachable!("Invalid subcommand"),
    }
}
