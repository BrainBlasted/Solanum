// app.rs
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


use gio::prelude::*;
use gtk::prelude::*;
use gtk_macros::*;

use glib::clone;

use gio::ApplicationFlags;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use gtk::subclass::prelude::*;

use once_cell::unsync::OnceCell;

use crate::config;
use crate::i18n::i18n;
use crate::window::SolanumWindow;

// Private struct for SolanumApplication, containing struct fields
#[derive(Debug)]
pub struct SolanumApplicationPriv {
    window: OnceCell<SolanumWindow>,
}

// Definite the GObject information for SolanumApplication
impl ObjectSubclass for SolanumApplicationPriv {
    const NAME: &'static str = "SolanumApplication";
    type ParentType = gtk::Application;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            window: OnceCell::new(),
        }
    }
}

// *Impl traits implement any vfuncs when subclassing a GObject
impl ObjectImpl for SolanumApplicationPriv {
    glib_object_impl!();
}


impl ApplicationImpl for SolanumApplicationPriv {
    fn activate(&self, _application: &gio::Application) {
        let window = self.window.get().expect("Could not get main window");
        window.show_all();
        window.present();
    }

    // Entry point for GApplication
    fn startup(&self, application: &gio::Application) {
        self.parent_startup(application);

        application.set_resource_base_path(Some("/io/gnome/Solanum/"));

        let app = application.clone().downcast::<SolanumApplication>().unwrap();
        let window = SolanumWindow::new(&app);
        window.set_title(&i18n("Solanum"));
        window.set_icon_name(Some(&config::APP_ID.to_owned()));
        self.window
            .set(window)
            .expect("Failed to init applciation window");

        app.setup_actions();
        app.setup_accels();
    }
}

impl GtkApplicationImpl for SolanumApplicationPriv {}

glib_wrapper! {
    pub struct SolanumApplication(
        Object<subclass::simple::InstanceStruct<SolanumApplicationPriv>,
        subclass::simple::ClassStruct<SolanumApplicationPriv>,
        SolanumApplicationClass>) @extends gio::Application, gtk::Application, @implements gio::ActionGroup, gio::ActionMap;

    match fn {
        get_type => || SolanumApplicationPriv::get_type().to_glib(),
    }
}

impl SolanumApplication {
    // Create the finalized, subclassed SolanumApplication
    pub fn new() -> Self {
        glib::Object::new(Self::static_type(), &[
            ("application-id", &config::APP_ID.to_owned()),
            ("flags", &ApplicationFlags::empty())
        ])
        .unwrap()
        .downcast()
        .unwrap()
    }

    fn get_main_window(&self) -> &SolanumWindow {
        let priv_ = SolanumApplicationPriv::from_instance(self);
        priv_.window.get().unwrap()
    }

    // Sets up gio::SimpleActions for the Application
    fn setup_actions(&self) {
        action!(self, "about", clone!(@strong self as app => move |_, _| {
            app.show_about();
        }));

        action!(self, "quit", clone!(@strong self as app => move |_, _| {
            app.quit();
        }));

        action!(self, "toggle-timer", clone!(@strong self as app => move |_, _| {
            let win = app.get_main_window();
            win.activate_action("toggle-timer", None);
        }));
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
        self.set_accels_for_action("win.menu", &["F10"]);
    }

    // About dialog
    fn show_about(&self) {
        let window = self.get_active_window();
        let authors = vec!["Christopher Davis <christopherdavis@gnome.org>"];
        let artists = vec!["Miredly Sound https://soundcloud.com/mired"];

        let dialog = gtk::AboutDialog::new();
        dialog.set_authors(&authors);
        dialog.set_artists(&artists);
        dialog.set_comments(Some(&i18n("A pomodoro timer for GNOME")));
        dialog.set_translator_credits(Some(&i18n("translator-credits")));
        dialog.set_license_type(gtk::License::Gpl30);
        dialog.set_logo_icon_name(Some(config::APP_ID));
        dialog.set_wrap_license(true);
        dialog.set_version(Some(config::VERSION));
        dialog.set_website(Some("https://www.patreon.com/chrisgnome"));
        dialog.set_website_label(Some(&i18n("Donate on Patreon")));
        dialog.set_copyright(Some(
            format!("\u{A9} {} Christopher Davis, et al.", config::COPYRIGHT).as_str(),
        ));

        if let Some(w) = window {
            dialog.set_transient_for(Some(&w));
            dialog.set_modal(true);
        }

        dialog.connect_response(move |d, _| {
            d.close();
        });
        dialog.show_all();
    }
}
