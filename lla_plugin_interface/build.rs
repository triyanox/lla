use std::io::Result;

fn main() -> Result<()> {
    #[cfg(feature = "regenerate-protobuf")]
    {
        use std::path::PathBuf;
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        println!("cargo:rerun-if-changed=src/plugin.proto");
        prost_build::Config::new()
            .out_dir(&out_dir)
            .compile_protos(&["src/plugin.proto"], &["src/"])
            .unwrap();

        std::fs::create_dir_all("src/generated")?;
        std::fs::copy(out_dir.join("lla_plugin.rs"), "src/generated/mod.rs")?;
    }
    Ok(())
}
