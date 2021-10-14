// preferences_window.rs
//
// Copyright 2021 Christopher Davis <christopherdavis@gnome.org>
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
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::subclass::prelude::*;

use once_cell::unsync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Solanum/preferences-window.ui")]
    pub struct SolanumPreferencesWindow {
        #[template_child]
        pub lap_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub short_break_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub long_break_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub session_count_spin: TemplateChild<gtk::SpinButton>,
        pub settings: OnceCell<gio::Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SolanumPreferencesWindow {
        const NAME: &'static str = "SolanumPreferencesWindow";
        type Type = super::SolanumPreferencesWindow;
        type ParentType = libadwaita::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SolanumPreferencesWindow {}
    impl WidgetImpl for SolanumPreferencesWindow {}
    impl WindowImpl for SolanumPreferencesWindow {}
    impl AdwWindowImpl for SolanumPreferencesWindow {}
    impl PreferencesWindowImpl for SolanumPreferencesWindow {}
}

glib::wrapper! {
    pub struct SolanumPreferencesWindow(ObjectSubclass<imp::SolanumPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, libadwaita::Window, libadwaita::PreferencesWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native,
        gtk::Root, gtk::ShortcutManager;
}

impl SolanumPreferencesWindow {
    pub fn new<W: glib::IsA<gtk::Window>>(parent: &W, settings: &gio::Settings) -> Self {
        let obj = glib::Object::new::<Self>(&[("transient-for", &Some(parent))]).unwrap();

        let imp = imp::SolanumPreferencesWindow::from_instance(&obj);

        settings.bind("lap-length", &*imp.lap_spin, "value").build();
        settings
            .bind("short-break-length", &*imp.short_break_spin, "value")
            .build();
        settings
            .bind("long-break-length", &*imp.long_break_spin, "value")
            .build();
        settings
            .bind(
                "sessions-until-long-break",
                &*imp.session_count_spin,
                "value",
            )
            .build();

        obj
    }
}
