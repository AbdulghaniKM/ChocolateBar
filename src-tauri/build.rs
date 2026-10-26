fn main() {
    tauri_build::build();
    embed_common_controls_manifest_outside_app_binary();
}

fn embed_common_controls_manifest_outside_app_binary() {
    let dependency = "/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' \
        version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'";
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg={dependency}");
    println!("cargo:rustc-link-arg-bins=/MANIFEST:NO");
}
