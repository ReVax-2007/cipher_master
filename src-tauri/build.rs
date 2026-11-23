fn main() {
  // tauri-build embeds the Common Controls v6 manifest only into binaries, so the
  // Windows test executable loads comctl32 v5 and dies with STATUS_ENTRYPOINT_NOT_FOUND
  // (TaskDialogIndirect, used by the dialog plugin). Embed it via the linker instead,
  // which applies to every target, test binaries included.
  let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
  tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
    .expect("failed to run tauri-build");

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
  if target_os == "windows" && target_env == "msvc" {
    let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
      .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
  }
}
