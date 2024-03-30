project('solanum', 'rust',
          version: '6.0.0',
    meson_version: '>= 0.59.0',
  default_options: [ 'warning_level=2',
                   ],
)

dependency('gtk4', version: '>= 4.13.0')
dependency('libadwaita-1', version: '>= 1.4.0')
dependency('gstreamer-1.0', version: '>= 1.20')
dependency('gstreamer-base-1.0', version: '>= 1.20')
dependency('gstreamer-audio-1.0', version: '>= 1.20')
dependency('gstreamer-play-1.0', version: '>= 1.20')
dependency('gstreamer-plugins-base-1.0', version: '>= 1.20')

i18n = import('i18n')
gnome = import('gnome')

solanum_prefix = get_option('prefix')
solanum_bindir = solanum_prefix / get_option('bindir')
solanum_localedir = solanum_prefix / get_option('localedir')
solanum_datadir = solanum_prefix / get_option('datadir')
solanum_pkgdatadir = solanum_datadir / meson.project_name()

copyright = '2022'
profile = ''
name_suffix = ''
version_suffix = ''

if get_option('profile') == 'development'
  profile = '.Devel'
  name_suffix = ' ☢'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: true).stdout().strip()
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

subdir('src')

meson.add_dist_script(
  'build-aux/meson/dist-vendor.sh',
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + meson.project_version(),
  meson.project_source_root()
)

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
