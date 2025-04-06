use std::path::Path;

fn handle_config_var(var: Option<&str>, key: &str) -> String {
    match var {
        Some(var) => var.to_owned(),
        None => {
            println!("cargo::warning=Build-time configuration variable \"{key}\" was missing. Use meson to compile the application instead of `cargo build`");
            "unknown".to_owned()
        }
    }
}

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config.rs");

    let app_id = handle_config_var(option_env!("APP_ID"), "APP_ID");
    let copyright = handle_config_var(option_env!("COPYRIGHT"), "COPYRIGHT");
    let pkg_data_dir = handle_config_var(option_env!("PKGDATADIR"), "PKGDATADIR");
    let version = handle_config_var(option_env!("VERSION"), "VERSION");
    let locale_dir = handle_config_var(option_env!("LOCALEDIR"), "LOCALEDIR");

    let module = format!(
        "pub static APP_ID: &str = \"{app_id}\";
pub static COPYRIGHT: &str = \"{copyright}\";
pub static PKGDATADIR: &str = \"{pkg_data_dir}\";
pub static VERSION: &str = \"{version}\";
pub static LOCALEDIR: &str = \"{locale_dir}\";
    "
    );

    std::fs::write(&dest_path, module).unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}
