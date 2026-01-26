extern crate prost_build;

fn main() {
    prost_build::compile_protos(
        &["src/cp_model.proto", "src/sat_parameters.proto"],
        &["src/"],
    )
    .unwrap();

    if std::env::var("DOCS_RS").is_err() {
        let ortools_prefix = std::env::var("ORTOOLS_PREFIX")
            .ok()
            .unwrap_or_else(|| "/opt/ortools".into());

        // Try to find abseil include path
        let abseil_prefix = std::env::var("ABSEIL_PREFIX")
            .ok()
            .or_else(|| {
                // Try homebrew location on macOS
                std::process::Command::new("brew")
                    .args(["--prefix", "abseil"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
            });

        // Try to find protobuf include path
        let protobuf_prefix = std::env::var("PROTOBUF_PREFIX")
            .ok()
            .or_else(|| {
                // Try homebrew location on macOS
                std::process::Command::new("brew")
                    .args(["--prefix", "protobuf"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
            });

        let mut build = cc::Build::new();
        build
            .cpp(true)
            .flags(["-std=c++17", "-DOR_PROTO_DLL="])
            .file("src/cp_sat_wrapper.cpp")
            .include([&ortools_prefix, "/include"].concat());

        if let Some(ref abseil) = abseil_prefix {
            build.include([abseil, "/include"].concat());
        }

        if let Some(ref protobuf) = protobuf_prefix {
            build.include([protobuf, "/include"].concat());
        }

        build.compile("cp_sat_wrapper.a");

        println!("cargo:rustc-link-lib=dylib=ortools");
        println!("cargo:rustc-link-search=native={}/lib", ortools_prefix);

        // Also link protobuf if available
        if let Some(ref protobuf) = protobuf_prefix {
            println!("cargo:rustc-link-lib=dylib=protobuf");
            println!("cargo:rustc-link-search=native={}/lib", protobuf);
        }
    }
}
