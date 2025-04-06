// Derived from https://gitlab.gnome.org/GNOME/loupe/-/blob/cc061312cdb1292420e86b8e0e132d6fa0ad4f52/src/meson.build
// Copyright (c) 2024 Sophie Herold
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub const APP_ID: &str = default_env(option_env!("APP_ID"), "org.gnome.Solanum.Devel");
pub const COPYRIGHT: &str = default_env(option_env!("COPYRIGHT"), "unknown");
pub const PKGDATADIR: &str = default_env(option_env!("PKGDATADIR"), "/usr/share/solanum/");
pub const VERSION: &str = default_env(option_env!("VERSION"), "unknown");
pub const LOCALEDIR: &str = default_env(option_env!("LOCALEDIR"), "/usr/share/locale/");

const fn default_env(v: Option<&'static str>, default: &'static str) -> &'static str {
    match v {
        Some(v) => v,
        None => default,
    }
}