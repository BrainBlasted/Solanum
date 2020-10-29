conf = configuration_data()
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('COPYRIGHT', copyright)
conf.set_quoted('VERSION', solanum_version)
conf.set_quoted('LOCALEDIR', solanum_localedir)
conf.set_quoted('PKGDATADIR', solanum_pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

rust_sources = files(
  'app.rs',
  'config.rs',
  'i18n.rs',
  'main.rs',
  'macros.rs',
  'templating.rs',
  'timer.rs',
  'window.rs',
)

sources = [cargo_sources, rust_sources]

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: solanum_bindir,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
