extern crate embed_resource;

use embed_manifest::{embed_manifest, manifest::ExecutionLevel, new_manifest};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let profile = std::env::var("PROFILE").unwrap();
    println!("cargo:rustc-cfg=feature=\"{}\"", profile);

    embed_manifest(new_manifest("repulse.manifest").requested_execution_level(ExecutionLevel::AsInvoker))
        .expect("unable to embed manifest file");
}
