scalable_dir = join_paths('hicolor', 'scalable', 'apps')
install_data(
  join_paths(scalable_dir, '@0@.svg'.format(app_id)),
  install_dir: join_paths(solanum_datadir, 'icons', scalable_dir),
)

symbolic_dir = join_paths('hicolor', 'symbolic', 'apps')
install_data(
  join_paths(symbolic_dir, 'org.gnome.Solanum-symbolic.svg'),
  install_dir: join_paths(solanum_datadir, 'icons', symbolic_dir),
  rename: '@0@-symbolic.svg'.format(app_id),
)

