project('solanum',
          version: '1.0.0',
    meson_version: '>= 0.50.0',
  default_options: [ 'warning_level=2',
                   ],
)

dependency('gtk+-3.0', version: '>= 3.24')
dependency('libhandy-1', version: '>= 0.90')

dependency('gstreamer-1.0', version: '>= 1.16')
dependency('gstreamer-base-1.0', version: '>= 1.16')
dependency('gstreamer-audio-1.0', version: '>= 1.16')
dependency('gstreamer-player-1.0', version: '>= 1.16')
dependency('gstreamer-plugins-base-1.0', version: '>= 1.16')

i18n = import('i18n')
gnome = import('gnome')

solanum_prefix = get_option('prefix')
solanum_bindir = join_paths(solanum_prefix, get_option('bindir'))
solanum_localedir = join_paths(solanum_prefix, get_option('localedir'))
solanum_datadir = join_paths(solanum_prefix, get_option('datadir'))
solanum_pkgdatadir = join_paths(solanum_datadir, meson.project_name())

app_id = 'org.gnome.Solanum'
copyright = '2020'

subdir('data')
subdir('po')

cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock'
)

subdir('src')

meson.add_install_script('build-aux/meson/postinstall.py')
meson.add_dist_script(
  'build-aux/meson/dist-vendor.sh',
  join_paths(meson.build_root(), 'meson-dist', meson.project_name() + '-' + meson.project_version()),
  meson.source_root()
)

