/// Test the new configuration system
use cv::configuration::AppConfiguration;

fn main() -> anyhow::Result<()> {
    println!("Testing new configuration system...");

    // Test loading configuration
    let config = AppConfiguration::load()?;

    println!("✅ Configuration loaded successfully!");
    println!("Data file: {}", config.paths.data_file);
    println!("Output directory: {}", config.paths.output_dir);
    println!("Server port: {}", config.server.port);
    println!("GitHub authenticated: {}", config.github.is_authenticated());
    println!("Cache TTL: {} seconds", config.github.cache_ttl);
    println!("Cache strategy: {:?}", config.github.cache_strategy);

    // Test builder pattern
    let custom_config = AppConfiguration::builder()
        .data_file("custom/data.json")
        .output_dir("custom/output")
        .server_port(9090)
        .dev_mode(true)
        .build();

    println!("\n✅ Builder pattern works!");
    println!("Custom data file: {}", custom_config.paths.data_file);
    println!("Custom output dir: {}", custom_config.paths.output_dir);
    println!("Custom port: {}", custom_config.server.port);
    println!("Dev mode: {}", custom_config.server.dev_mode);

    // Test output paths
    let paths = config.output_paths();
    println!("\n✅ Output paths work!");
    println!("HTML path: {}", paths.html);
    println!("PDF path: {}", paths.pdf);

    // Test data field parsing
    println!("\n✅ Data field parsing works!");
    let public_fields = config.data.public_fields_list();
    let db_fields = config.data.database_fields_list();
    println!("Public fields: {:?}", public_fields);
    println!("Database fields: {:?}", db_fields);
    println!("Is 'name' public: {}", config.data.is_public_field("name"));
    println!(
        "Is 'secret' public: {}",
        config.data.is_public_field("secret")
    );

    println!("\n🎉 All configuration tests passed!");

    Ok(())
}
