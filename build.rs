use std::error::Error;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/api/queries/*.graphql");
    println!("cargo:rerun-if-changed=src/api/queries/schema.graphql");

    // Check if graphql_client_cli is installed
    let check_graphql_cli = Command::new("cargo")
        .args(&["install", "--list"])
        .output()?;

    let output = String::from_utf8(check_graphql_cli.stdout)?;

    if !output.contains("graphql_client_cli") {
        println!("cargo:warning=graphql_client_cli is not installed. Installing...");

        // Install graphql_client_cli
        let install_result = Command::new("cargo")
            .args(&["install", "graphql_client_cli"])
            .status()?;

        if !install_result.success() {
            return Err("Failed to install graphql_client_cli".into());
        }
    }

    // Ensure the schema path exists
    let schema_path = Path::new("src/api/queries/schema.graphql");
    if !schema_path.exists() {
        return Err(format!("Schema file does not exist: {:?}", schema_path).into());
    }

    Ok(())
}
