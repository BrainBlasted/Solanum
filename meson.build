project('solanum', 'rust',
          version: '3.0.1',
    meson_version: '>= 0.50.0',
  default_options: [ 'warning_level=2',
                   ],
)

dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1')
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

copyright = '2021'
profile = ''
name_suffix = ''
version_suffix = ''

if get_option('profile') == 'development'
  profile = '.Devel'
  name_suffix = ' ☢'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
endif

app_id = 'org.gnome.Solanum@0@'.format(profile)
solanum_version = meson.project_version() + version_suffix

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

