cargo = find_program('cargo', required: true)

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_env = environment()
cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')
cargo_env.set('APP_ID', app_id)
cargo_env.set('COPYRIGHT', copyright)
cargo_env.set('VERSION', solanum_version)
cargo_env.set('LOCALEDIR', solanum_localedir)
cargo_env.set('PKGDATADIR', solanum_pkgdatadir)

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: solanum_bindir,
  depends: resources,
  env: cargo_env,
  command: [
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', 'src' / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)
