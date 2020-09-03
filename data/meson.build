desktop_conf = configuration_data()
desktop_conf.set('APP_ID', app_id)
desktop_file = i18n.merge_file(
  'desktop',
  input: configure_file(
    input: 'io.gnome.Solanum.desktop.in.in',
    output: 'io.gnome.Solanum.desktop.in',
    configuration: desktop_conf
  ),
  output: 'io.gnome.Solanum.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(solanum_datadir, 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

appstream_file = i18n.merge_file(
  input: 'io.gnome.Solanum.appdata.xml.in',
  output: 'io.gnome.Solanum.appdata.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(solanum_datadir, 'appdata')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util,
    args: ['validate', appstream_file]
  )
endif

install_data('io.gnome.Solanum.gschema.xml',
  install_dir: join_paths(solanum_datadir, 'glib-2.0', 'schemas')
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif

gnome.compile_resources(
  'solanum',
  'solanum.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: solanum_pkgdatadir,
)

subdir('icons')
