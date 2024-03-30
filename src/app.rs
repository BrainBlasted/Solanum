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

use gtk::prelude::*;
use gtk::{gio, glib};

use glib::WeakRef;

use glib::subclass::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita::subclass::prelude::*;

use once_cell::unsync::OnceCell;

use crate::config;
use crate::i18n::i18n;
use crate::preferences_window::SolanumPreferencesWindow;
use crate::window::SolanumWindow;

mod imp {
    use super::*;

    // Private struct for SolanumApplication, containing struct fields
    #[derive(Debug)]
    pub struct SolanumApplication {
        pub window: OnceCell<WeakRef<SolanumWindow>>,
        pub settings: gio::Settings,
    }

    // Definite the GObject information for SolanumApplication
    #[glib::object_subclass]
    impl ObjectSubclass for SolanumApplication {
        const NAME: &'static str = "SolanumApplication";
        type Type = super::SolanumApplication;
        type ParentType = libadwaita::Application;

        fn new() -> Self {
            Self {
                window: OnceCell::new(),
                settings: gio::Settings::new("org.gnome.Solanum"),
            }
        }
    }

    // *Impl traits implement any vfuncs when subclassing a GObject
    impl ObjectImpl for SolanumApplication {}

    impl ApplicationImpl for SolanumApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = application.get_main_window();
            window.present();
        }

        // Entry point for GApplication
        fn startup(&self) {
            self.parent_startup();

            let application = self.obj();

            let window = SolanumWindow::new(&*application);
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
    impl AdwApplicationImpl for SolanumApplication {}
}

glib::wrapper! {
    pub struct SolanumApplication(ObjectSubclass<imp::SolanumApplication>)
        @extends gio::Application, gtk::Application, libadwaita::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SolanumApplication {
    // Create the finalized, subclassed SolanumApplication
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/org/gnome/Solanum")
            .build()
    }

    fn get_main_window(&self) -> SolanumWindow {
        self.imp().window.get().unwrap().clone().upgrade().unwrap()
    }

    pub fn gsettings(&self) -> &gio::Settings {
        &self.imp().settings
    }

    // Sets up gio::SimpleActions for the Application
    fn setup_actions(&self) {
        let actions = [
            gio::ActionEntryBuilder::new("about")
                .activate(|app: &Self, _, _| app.show_about())
                .build(),
            gio::ActionEntryBuilder::new("preferences")
                .activate(|app: &Self, _, _| app.show_preferences())
                .build(),
            gio::ActionEntryBuilder::new("quit")
                .activate(|app: &Self, _, _| app.quit())
                .build(),
            gio::ActionEntryBuilder::new("toggle-timer")
                .activate(|app: &Self, _, _| {
                    let win: gtk::Widget = app.get_main_window().upcast();
                    let _ = win.activate_action("win.toggle-timer", None);
                })
                .build(),
            gio::ActionEntryBuilder::new("skip")
                .activate(|app: &Self, _, _| {
                    let win: gtk::Widget = app.get_main_window().upcast();
                    let _ = win.activate_action("win.skip", None);
                })
                .build(),
        ];

        self.add_action_entries(actions);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.preferences", &["<Primary>comma"]);
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
    }

    // About dialog
    fn show_about(&self) {
        let window = self.get_main_window();
        let developers = &["Christopher Davis <christopherdavis@gnome.org>"];

        let about = libadwaita::AboutWindow::from_appdata(
            &format!("/org/gnome/Solanum/{}.appdata.xml", config::APP_ID),
            if !config::APP_ID.ends_with("Devel") {
                Some(config::VERSION)
            } else {
                None
            },
        );

        about.set_translator_credits(&i18n("translator-credits"));
        about.set_version(config::VERSION);
        about.set_copyright(&format!(
            "\u{A9} {} Christopher Davis, et al.",
            config::COPYRIGHT
        ));
        about.set_developers(developers);

        about.add_link(
            &i18n("_Donate on Patreon"),
            "https://www.patreon.com/chrisgnome",
        );
        about.add_link(
            &i18n("_Sponsor on GitHub"),
            "https://github.com/sponsors/BrainBlasted/",
        );

        about.add_credit_section(
            Some(&i18n("Icon by")),
            &["Tobias Bernard https://tobiasbernard.com"],
        );
        about.add_credit_section(
            Some(&i18n("Sound by")),
            &["Miredly Sound https://soundcloud.com/mired"],
        );

        about.add_acknowledgement_section(
            Some(&i18n("Supported by")),
            &[
                "Willo Vincent",
                "Sage Rosen",
                "refi64",
                "Patrons and GitHub Sponsors",
            ],
        );

        about.set_transient_for(Some(&window));
        about.set_modal(true);

        about.present();
    }

    fn show_preferences(&self) {
        let imp = self.imp();
        let window = self.get_main_window();
        let preferences_window = SolanumPreferencesWindow::new(&window, &imp.settings);
        preferences_window.present();
    }
}
