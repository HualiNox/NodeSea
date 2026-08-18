use std::{
    env,
    fs::{self},
    path::Path,
};

fn json_string(value: impl AsRef<str>) -> String {
    format!(
        "\"{}\"",
        value.as_ref().replace('\\', "\\\\").replace('"', "\\\"")
    )
}

fn main() {
    // Build the CMake project and get the output directory
    let dst = cmake::Config::new(".")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("build_tests", "OFF")
        .define("build_examples", "OFF")
        .define("build_tools", "OFF")
        .build();

    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();
    let cmake_include = dst.join("include");
    let cxxbridge_include = out_dir.join("cxxbridge/include");
    let project_include = Path::new("include");
    let boost_source = out_dir.join("build/_deps/boost_src-src");
    let mut boost_include_dirs = Vec::new();
    let boost_libs = boost_source.join("libs");
    if let Ok(entries) = fs::read_dir(&boost_libs) {
        for entry in entries.flatten() {
            let include_dir = entry.path().join("include");
            if include_dir.is_dir() {
                boost_include_dirs.push(include_dir);
            }
        }
    }

    // Build the C++ bridge. The include directory comes from CMake's install
    // prefix and the Boost source fetched by CMake, so it does not depend on
    // a system/Homebrew Boost installation.
    cxx_build::CFG.include_prefix = "";
    let mut bridge = cxx_build::bridge("src/ffi.rs");

    // Add all .cpp files from the cpp directory
    if let Ok(entries) = fs::read_dir("cpp") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "cpp") {
                bridge.file(path);
            }
        }
    }

    // Include directories
    bridge
        .include(project_include)
        .include(&cmake_include)
        .include(&cxxbridge_include)
        .define("TORRENT_USE_OPENSSL", None)
        .define("TORRENT_ABI_VERSION", Some("2"))
        .define("BOOST_ASIO_ENABLE_CANCELIO", None)
        .define("BOOST_ASIO_NO_DEPRECATED", None)
        .define("BOOST_SYSTEM_USE_UTF8", None)
        .define("_SILENCE_CXX17_ALLOCATOR_VOID_DEPRECATION_WARNING", None)
        .std("c++20");
    for include_dir in &boost_include_dirs {
        bridge.include(include_dir);
    }
    bridge.compile("nodesea-bt-ffi");

    // Keep one editor-independent compilation database. Both VSCode and Zed
    // can consume the repository-root compile_commands.json through clangd.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    let source = Path::new(&manifest_dir).join("cpp/engine.cpp");
    let compile_commands = workspace_root.join("compile_commands.json");
    let mut include_flags = vec![Path::new(&manifest_dir).join("include"), cmake_include];
    let editor_include = Path::new(&manifest_dir).join(".generated");
    include_flags.push(editor_include.clone());
    include_flags.push(cxxbridge_include);
    include_flags.extend(boost_include_dirs);
    let arguments = include_flags
        .iter()
        .map(|path| json_string(format!("-I{}", path.to_string_lossy())))
        .collect::<Vec<_>>()
        .join(", ");
    let arguments = format!("{}, {}", json_string("-DTORRENT_USE_OPENSSL"), arguments);
    let command = format!(
        "{{\n  \"directory\": {},\n  \"file\": {},\n  \"arguments\": [\"c++\", \"-std=c++20\", {}, {}]\n}}\n",
        json_string(&manifest_dir),
        json_string(source.to_string_lossy()),
        arguments,
        json_string(source.to_string_lossy()),
    );
    fs::write(compile_commands, format!("[{}]", command)).unwrap();

    // Keep a stable copy for clangd. The real CXX header lives under Cargo's
    // hash-based OUT_DIR, which is not practical to reference from an editor.
    let generated_header =
        Path::new(&env::var("OUT_DIR").unwrap()).join("cxxbridge/include/src/ffi.rs.h");
    let editor_header = editor_include.join("src/ffi.rs.h");
    fs::create_dir_all(editor_header.parent().unwrap()).unwrap();
    fs::copy(generated_header, editor_header).unwrap();

    // Link the C++ library and its internal dependencies
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );

    println!("cargo:rustc-link-lib=static=torrent-rasterbar");
    println!("cargo:rustc-link-lib=static=datachannel-static");
    println!("cargo:rustc-link-lib=static=usrsctp");
    println!("cargo:rustc-link-lib=static=juice-static");

    // Dynamically retrieve OpenSSL library paths discovered by CMake.
    // This completely avoids hardcoding system or Homebrew-specific paths.
    let cmake_cache = out_dir.join("build/CMakeCache.txt");
    if let Ok(cache_content) = fs::read_to_string(&cmake_cache) {
        for line in cache_content.lines() {
            if let Some(lib_path) = line
                .strip_prefix("OPENSSL_CRYPTO_LIBRARY:FILEPATH=")
                .or_else(|| line.strip_prefix("OPENSSL_SSL_LIBRARY:FILEPATH="))
            {
                let path = Path::new(lib_path);
                if let Some(parent) = path.parent() {
                    println!("cargo:rustc-link-search=native={}", parent.display());
                }
            }
        }
    }

    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=SystemConfiguration");
        println!("cargo:rustc-link-lib=framework=Security");
    }

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=cpp");
    println!("cargo:rerun-if-changed=include");
}
