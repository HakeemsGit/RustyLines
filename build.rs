use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Retrieve environment variables set by Cargo
    let target = env::var("TARGET").expect("TARGET environment variable not set");
    let profile = env::var("PROFILE").expect("PROFILE environment variable not set");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Determine the binary name based on the target OS
    let binary_name = if target.contains("windows") {
        "rustylines.exe"
    } else {
        "rustylines"
    };

    // Define the path to the built binary within the target directory
    let built_binary = Path::new(&manifest_dir)
        .join("target")
        .join(&target)
        .join(&profile)
        .join(binary_name);

    // Define the destination path in the main project directory
    let destination = Path::new(&manifest_dir).join(binary_name);

    // Check if the built binary exists
    if built_binary.exists() {
        // Copy the binary to the destination
        fs::copy(&built_binary, &destination)
            .expect("Failed to copy binary to the main directory.");
        
        // Emit a warning message to Cargo to inform the user
        println!("cargo:warning=Binary copied to the main directory.");
    } else {
        // Emit a warning if the binary was not found
        println!("cargo:warning=Built binary not found at {:?}", built_binary);
    }
}
