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
use glib::WeakRef;

use gio::ApplicationFlags;
use glib::subclass;
use glib::subclass::prelude::*;
use gtk::subclass::prelude::*;

use once_cell::unsync::OnceCell;

use crate::config;
use crate::i18n::i18n;
use crate::window::SolanumWindow;

mod imp {
    use super::*;

    // Private struct for SolanumApplication, containing struct fields
    #[derive(Debug)]
    pub struct SolanumApplication {
        pub window: OnceCell<WeakRef<SolanumWindow>>,
    }

    // Definite the GObject information for SolanumApplication
    impl ObjectSubclass for SolanumApplication {
        const NAME: &'static str = "SolanumApplication";
        type Type = super::SolanumApplication;
        type ParentType = gtk::Application;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                window: OnceCell::new(),
            }
        }
    }

    // *Impl traits implement any vfuncs when subclassing a GObject
    impl ObjectImpl for SolanumApplication {}

    impl ApplicationImpl for SolanumApplication {
        fn activate(&self, application: &Self::Type) {
            let window = application.get_main_window();
            window.show();
            window.present();
        }

        // Entry point for GApplication
        fn startup(&self, application: &Self::Type) {
            self.parent_startup(application);

            application.set_resource_base_path(Some("/org/gnome/Solanum/"));

            let window = SolanumWindow::new(application);
            window.set_title(Some(&i18n("Solanum")));
            window.set_icon_name(Some(&config::APP_ID.to_owned()));
            self.window
                .set(window.downgrade())
                .expect("Failed to init application window");

            application.setup_actions();
            application.setup_accels();
        }
    }

    impl GtkApplicationImpl for SolanumApplication {}
}

glib::wrapper! {
    pub struct SolanumApplication(ObjectSubclass<imp::SolanumApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SolanumApplication {
    // Create the finalized, subclassed SolanumApplication
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &config::APP_ID.to_owned()),
            ("flags", &ApplicationFlags::empty()),
        ])
        .unwrap()
    }

    fn get_main_window(&self) -> SolanumWindow {
        let imp = imp::SolanumApplication::from_instance(self);
        imp.window.get().unwrap().clone().upgrade().unwrap()
    }

    // Sets up gio::SimpleActions for the Application
    fn setup_actions(&self) {
        action!(
            self,
            "about",
            clone!(@strong self as app => move |_, _| {
                app.show_about();
            })
        );

        action!(
            self,
            "quit",
            clone!(@strong self as app => move |_, _| {
                app.quit();
            })
        );

        action!(
            self,
            "toggle-timer",
            clone!(@strong self as app => move |_, _| {
                let win = app.get_main_window();
                win.activate_action("toggle-timer", None);
            })
        );

        action!(
            self,
            "skip",
            clone!(@strong self as app => move |_, _| {
                let win = app.get_main_window();
                win.activate_action("skip", None);
            })
        );
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
        self.set_accels_for_action("win.menu", &["F10"]);
    }

    // About dialog
    fn show_about(&self) {
        let window = self.get_active_window();
        let authors = vec!["Christopher Davis <christopherdavis@gnome.org>".to_string()];
        let artists = vec![
            "Tobias Bernard https://tobiasbernard.com".to_string(),
            "Miredly Sound https://soundcloud.com/mired".to_string(),
        ];

        let dialog = gtk::AboutDialogBuilder::new()
            .authors(authors)
            .artists(artists)
            .comments(&i18n("A pomodoro timer for GNOME"))
            // Translators: Replace "translator-credits" with your names, one name per line
            .translator_credits(&i18n("translator-credits"))
            .license_type(gtk::License::Gpl30)
            .logo_icon_name(config::APP_ID)
            .wrap_license(true)
            .version(config::VERSION)
            .website("https://www.patreon.com/chrisgnome")
            .website_label(&i18n("Donate on Patreon"))
            .copyright(format!("\u{A9} {} Christopher Davis, et al.", config::COPYRIGHT).as_str())
            .build();

        if let Some(w) = window {
            dialog.set_transient_for(Some(&w));
            dialog.set_modal(true);
        }

        dialog.show();
    }
}
