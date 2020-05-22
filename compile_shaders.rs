use std::{env, path::Path, process::Command};

fn find_glsl_compiler() -> String {
    let search_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    let output = Command::new(search_cmd)
        .arg("glslc")
        .output()
        .expect("Failed to retrieve shader compiler path!");

    let compiler_path = String::from_utf8(output.stdout).unwrap();
    compiler_path.trim().to_string()
}

fn compile_shader(compiler_cmd: &str, input_file: &str, output_file: &str, err_msg: &str) {
    let script_dir = env::current_dir().expect("Failed to obtain current directory!");
    Command::new(compiler_cmd)
        .arg(
            script_dir
                .join(Path::new("src/shaders/src"))
                .join(input_file)
                .to_str()
                .unwrap(),
        )
        .arg("-o")
        .arg(
            script_dir
                .join(Path::new("src/shaders"))
                .join(output_file)
                .to_str()
                .unwrap(),
        )
        .output()
        .expect(err_msg);
}

/// Shader build script - crate [cargo-script](https://crates.io/crates/cargo-script) is used to run this from CLI.
/// NOTE: run `cargo script compile_shaders.rs` from the project root path. If CARGO_MANIFEST_DIR would be accessible from scripts path resolving would be easily fixed :(
fn main() {
    let compiler_cmd = find_glsl_compiler();
    compile_shader(
        &compiler_cmd,
        "shader.vert",
        "vert.spv",
        "Failed to compile vertex shader!",
    );
    compile_shader(
        &compiler_cmd,
        "shader.frag",
        "frag.spv",
        "Failed to compile fragment shader!",
    );
    println!("Shader compilation successful.")
}
