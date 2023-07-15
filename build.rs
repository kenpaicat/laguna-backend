use std::{fs, env};

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=assets");
    if let Ok(_) = env::var("CARGO_FEATURE_DOX") {
        println!("cargo:warning=Copying logos to documentation.");
        fs::copy("assets/logo.png", "target/doc/logo.png")
            .expect("Failed to copy crate logo when building documentation.");
        fs::copy("assets/favicon.ico", "target/doc/favicon.ico")
            .expect("Failed to copy crate favicon when building documentation.");
    }
}
