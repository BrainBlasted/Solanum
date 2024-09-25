desktop_conf = configuration_data()
desktop_conf.set('APP_ID', app_id)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'org.gnome.Solanum.desktop.in.in',
    output: 'org.gnome.Solanum.desktop.in',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(app_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: solanum_datadir / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

appstream_file = i18n.merge_file(
  input: configure_file(
    input: 'org.gnome.Solanum.appdata.xml.in.in',
    output: 'org.gnome.Solanum.appdata.xml.in',
    configuration: desktop_conf,
  ),
  output: '@0@.appdata.xml'.format(app_id),
  po_dir: '../po',
  install: true,
  install_dir: solanum_datadir / 'metainfo'
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('Validate appstream file', appstreamcli,
    args: ['validate', '--no-net', '--explain', appstream_file]
  )
endif

install_data('org.gnome.Solanum.gschema.xml',
  install_dir: solanum_datadir / 'glib-2.0' / 'schemas'
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif

blueprints = custom_target('blueprints',
  input: files(
    'gtk/help-overlay.blp',
    'ui/preferences-window.blp',
    'ui/window.blp',
  ),
  output: '.',
  build_always_stale: true,
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

resources = gnome.compile_resources(
  'solanum',
  configure_file(
    input: 'solanum.gresource.xml.in',
    output: 'solanum.gresource.xml',
    configuration: desktop_conf,
  ),
  dependencies: [ blueprints, appstream_file ],
  gresource_bundle: true,
  install: true,
  install_dir: solanum_pkgdatadir,
)

subdir('icons')
