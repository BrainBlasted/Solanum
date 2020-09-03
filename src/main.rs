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


#[macro_use]
extern crate glib;
extern crate gtk;
extern crate gio;

#[macro_use]
extern crate gtk_macros;

use gettextrs::*;
use gio::prelude::*;
use gtk::prelude::*;

mod app;
mod config;
mod i18n;
#[macro_use]
mod macros;
mod timer;
mod window;

use crate::app::SolanumApplication;

// Entry point for the application
fn main() {
    // Initiialize gtk, gstreamer, and libhandy.
    gtk::init().expect("Failed to initialize gstreamer");
    libhandy::init();
    gstreamer::init().expect("Failed to initialize gstreamer");

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

    // Set up CSS
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/io/gnome/Solanum/style.css");
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().unwrap(),
        &provider,
        600,
    );

    let app = SolanumApplication::new();

    let ret = app.run(&std::env::args().collect::<Vec<_>>());
    std::process::exit(ret);
}

