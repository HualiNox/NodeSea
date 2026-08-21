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

const PRODUCTION_BRIDGE_SOURCES: &[&str] = &[
    "src/ffi.rs",
    "src/ffi/dht.rs",
    "src/ffi/session.rs",
    "src/ffi/torrent.rs",
];

// Native C++ sources compiled into the Rust FFI library.
const NATIVE_SOURCES: &[&str] = &["cpp/engine.cpp", "cpp/alert_parser.cpp", "cpp/helper.cpp"];

fn main() {
    // CMake owns libtorrent's feature policy and derives this profile from
    // Cargo's PROFILE, OPT_LEVEL, and DEBUG environment variables.
    let mut cmake_config = cmake::Config::new(".");
    let native_profile = cmake_config.get_profile().to_owned();
    let dst = cmake_config.build();
    let debug_native_build = native_profile == "Debug";

    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();
    let cmake_include = dst.join("include");
    let cxxbridge_include = out_dir.join("cxxbridge/include");
    let project_include = Path::new("include");

    // Build the C++ bridge against CMake's installed libtorrent and Boost
    // headers, without depending on a system/Homebrew Boost installation.
    cxx_build::CFG.include_prefix = "";
    let mut bridge = cxx_build::bridges(PRODUCTION_BRIDGE_SOURCES.iter().copied());

    for source in NATIVE_SOURCES {
        bridge.file(source);
    }

    // Include directories
    bridge
        .include(project_include)
        .include(&cmake_include)
        .include(&cxxbridge_include)
        .define("TORRENT_USE_OPENSSL", None)
        .define("TORRENT_ABI_VERSION", Some("2"))
        .define("TORRENT_USE_RTC", Some("0"))
        .define("TORRENT_USE_I2P", Some("0"))
        .define("BOOST_ASIO_ENABLE_CANCELIO", None)
        .define("BOOST_ASIO_NO_DEPRECATED", None)
        .define("BOOST_SYSTEM_USE_UTF8", None)
        .define("_SILENCE_CXX17_ALLOCATOR_VOID_DEPRECATION_WARNING", None)
        .std("c++20");
    if debug_native_build {
        bridge.define("TORRENT_USE_ASSERTS", None);
    }
    bridge.compile("nodesea-bt-ffi");

    // Keep one editor-independent compilation database. Both VSCode and Zed
    // can consume the repository-root compile_commands.json through clangd.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    let compile_commands = workspace_root.join("compile_commands.json");
    let engine_source = Path::new(&manifest_dir).join("cpp/engine.cpp");
    let alert_parser_source = Path::new(&manifest_dir).join("cpp/alert_parser.cpp");
    let helper_source = Path::new(&manifest_dir).join("cpp/helper.cpp");
    let installed_include = cmake_include;
    let editor_include = Path::new(&manifest_dir).join(".generated");
    let debug_assert_argument = if debug_native_build {
        format!(", {}", json_string("-DTORRENT_USE_ASSERTS"))
    } else {
        String::new()
    };
    let common_arguments = format!(
        "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}{}",
        json_string("-DTORRENT_USE_OPENSSL"),
        json_string("-DTORRENT_ABI_VERSION=2"),
        json_string("-DTORRENT_USE_RTC=0"),
        json_string("-DTORRENT_USE_I2P=0"),
        json_string("-DBOOST_ASIO_ENABLE_CANCELIO"),
        json_string("-DBOOST_ASIO_NO_DEPRECATED"),
        json_string("-DBOOST_SYSTEM_USE_UTF8"),
        json_string("-D_SILENCE_CXX17_ALLOCATOR_VOID_DEPRECATION_WARNING"),
        json_string(format!(
            "-I{}",
            Path::new(&manifest_dir).join("include").display()
        )),
        json_string(format!("-I{}", installed_include.display())),
        json_string(format!("-I{}", editor_include.display())),
        json_string(format!("-I{}", cxxbridge_include.display())),
        debug_assert_argument,
    );
    let engine_command = format!(
        "{{\n  \"directory\": {},\n  \"file\": {},\n  \"arguments\": [\"c++\", \"-std=c++20\", {}, {}]\n}}\n",
        json_string(&manifest_dir),
        json_string(engine_source.to_string_lossy()),
        common_arguments,
        json_string(engine_source.to_string_lossy()),
    );
    let helper_command = format!(
        "{{\n  \"directory\": {},\n  \"file\": {},\n  \"arguments\": [\"c++\", \"-std=c++20\", {}, {}]\n}}\n",
        json_string(&manifest_dir),
        json_string(helper_source.to_string_lossy()),
        common_arguments,
        json_string(helper_source.to_string_lossy()),
    );
    let alert_parser_command = format!(
        "{{\n  \"directory\": {},\n  \"file\": {},\n  \"arguments\": [\"c++\", \"-std=c++20\", {}, {}]\n}}\n",
        json_string(&manifest_dir),
        json_string(alert_parser_source.to_string_lossy()),
        common_arguments,
        json_string(alert_parser_source.to_string_lossy()),
    );
    fs::write(
        compile_commands,
        format!("[{engine_command},{alert_parser_command},{helper_command}]"),
    )
    .unwrap();

    // Keep stable copies for clangd. The real CXX headers live under Cargo's
    // hash-based OUT_DIR, which is not practical to reference from an editor.
    let generated_include = Path::new(&env::var("OUT_DIR").unwrap()).join("cxxbridge/include");
    for source in PRODUCTION_BRIDGE_SOURCES {
        let generated_header = generated_include.join(format!("{source}.h"));
        let editor_header = editor_include.join(format!("{source}.h"));
        fs::create_dir_all(editor_header.parent().unwrap()).unwrap();
        fs::copy(generated_header, editor_header).unwrap();
    }

    // Link the C++ library and its internal dependencies
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );

    println!("cargo:rustc-link-lib=static=torrent-rasterbar");

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

    let rerun_paths = [
        // Rust FFI implementation trees.
        "src/ffi.rs",
        "src/ffi",
        "src/types",
        // Production C++ bridge sources and headers.
        "cpp/engine.cpp",
        "cpp/alert_parser.cpp",
        "cpp/helper.cpp",
        "include/nodesea_bt/engine.hpp",
        "include/nodesea_bt/alert_parser.hpp",
        "include/nodesea_bt/helper.hpp",
    ];
    for path in rerun_paths {
        println!("cargo:rerun-if-changed={path}");
    }
}
