fn main() {
    // Compile GLib resources (icons) into a .gresource binary file.
    // This file will be included in the binary using gio::resources_register_include! macro.
    let manifest_dir = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"),
    );
    let out_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is not set"));

    // Generate binary .gresource file from the resource XML
    let resource_xml = manifest_dir.join("data/icons.gresource.xml");
    let gresource_file = out_dir.join("icons.gresource");

    let output = std::process::Command::new("glib-compile-resources")
        .arg(&resource_xml)
        .arg(format!(
            "--sourcedir={}",
            manifest_dir.join("data/icons").display()
        ))
        .arg(format!("--target={}", gresource_file.display()))
        .output()
        .expect("Failed to run glib-compile-resources");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("glib-compile-resources failed: {stderr}");
    }

    // Tell Cargo to re-run if the resource files change
    println!("cargo:rerun-if-changed={}", resource_xml.display());
    println!("cargo:rerun-if-changed=build.rs");
    // Also watch the icon files
    for entry in std::fs::read_dir(manifest_dir.join("data/icons")).unwrap() {
        let entry = entry.unwrap();
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    // Generate the Windows icon and version resource, then attach it
    // specifically to the application binary. `winresource::compile()` emits
    // `rustc-link-lib`, which Cargo routes only to this package's library now
    // that `src/lib.rs` exists; the resource would therefore be absent from
    // `tributary.exe`. `embed_resource::compile_for()` emits the bin-scoped
    // linker directive required by a mixed library/binary package.
    #[cfg(target_os = "windows")]
    {
        let output_dir =
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is not set"));
        let resource_file = output_dir.join("tributary-resource.rc");
        let icon = manifest_dir.join("data/tributary.ico");

        let mut res = winresource::WindowsResource::new();
        res.set_icon(
            icon.to_str()
                .expect("Windows resource icon path is not valid UTF-8"),
        );
        res.set("ProductName", "Tributary");
        res.set("FileDescription", "Tributary");
        res.set("LegalCopyright", "Copyright © 2026 Tributary Contributors");
        res.write_resource_file(&resource_file)
            .expect("Failed to generate Windows resources");

        embed_resource::compile_for(&resource_file, ["tributary"], embed_resource::NONE)
            .manifest_required()
            .expect("Failed to compile Windows resources for tributary.exe");
    }
}
