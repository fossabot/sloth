const OUTPUT_DIRECTORY: &str = ".sloth";

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().count() > 1 {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "too many arguments").into(),
        );
    }

    let package_configuration =
        app::PackageConfiguration::from_json(&std::fs::read_to_string("package.json")?)?;
    let object_file_storage = infra::FileStorage::new(OUTPUT_DIRECTORY, "bc");

    app::PackageBuilder::new(
        &app::ModuleCompiler::new(
            &app::ModulePathConverter::new(&package_configuration),
            &infra::FileStorage::new("src", "sl"),
            &object_file_storage,
            &infra::FileStorage::new(OUTPUT_DIRECTORY, "json"),
        ),
        &infra::Linker::new(
            &package_configuration,
            std::env::var("SLOTH_ROOT")?,
            &object_file_storage,
        ),
    )
    .build()
}
