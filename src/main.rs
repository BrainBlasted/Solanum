// main.rs
//
// Copyright 2020 Christopher Davis <christopherdavis@gnome.org>
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

use gettextrs::*;
use gio::prelude::*;

mod app;
mod config;
mod i18n;
mod preferences_window;
mod timer;
mod window;

use crate::app::SolanumApplication;

// Entry point for the application
fn main() -> glib::ExitCode {
    // Initiialize gtk, gstreamer, and libhandy.
    gtk::init().expect("Failed to initialize gtk");
    gstreamer::init().expect("Failed to initialize gstreamer");
    libadwaita::init();

    // Set up translations
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("solanum", config::LOCALEDIR);
    textdomain("solanum");

    // Register resources so we can integrate things like UI files, CSS, and icons
    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/solanum.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    // Set the name shown in desktop environments
    glib::set_application_name("Solanum");
    glib::set_program_name(Some("solanum"));

    let app = SolanumApplication::new();

    app.run()
}
