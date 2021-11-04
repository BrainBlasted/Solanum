// window.rs
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

use glib::clone;
use gtk::CompositeTemplate;
use gtk_macros::*;

use glib::subclass;
use glib::subclass::prelude::*;
use gtk::prelude::IsA;
use gtk::subclass::prelude::*;
use libadwaita::subclass::prelude::*;

use std::cell::Cell;

use crate::app::SolanumApplication;
use crate::config;
use crate::i18n::*;
use crate::timer::{LapType, Timer};

static CHIME_URI: &str = "resource:///org/gnome/Solanum/chime.ogg";
static BEEP_URI: &str = "resource:///org/gnome/Solanum/beep.ogg";

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/Solanum/window.ui")]
    pub struct SolanumWindow {
        pub pomodoro_count: Cell<u32>,
        pub timer: Timer,
        pub player: gstreamer_player::Player,
        #[template_child]
        pub lap_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub timer_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub timer_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SolanumWindow {
        const NAME: &'static str = "SolanumWindow";
        type Type = super::SolanumWindow;
        type ParentType = libadwaita::ApplicationWindow;

        fn new() -> Self {
            Self {
                pomodoro_count: Cell::new(1),
                timer: Timer::new(),
                player: gstreamer_player::Player::new(None, None),
                lap_label: TemplateChild::default(),
                timer_label: TemplateChild::default(),
                timer_button: TemplateChild::default(),
                menu_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // We don't need to override any vfuncs here, but since they're superclasses
    // we need to declare the blank impls
    impl ObjectImpl for SolanumWindow {}
    impl WidgetImpl for SolanumWindow {}
    impl WindowImpl for SolanumWindow {}
    impl ApplicationWindowImpl for SolanumWindow {}
    impl AdwApplicationWindowImpl for SolanumWindow {}
}

glib::wrapper! {
    pub struct SolanumWindow(ObjectSubclass<imp::SolanumWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libadwaita::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SolanumWindow {
    pub fn new<P: IsA<gtk::Application> + glib::value::ToValue>(app: &P) -> Self {
        let win = glib::Object::new::<Self>(&[("application", app)])
            .expect("Failed to create SolanumWindow");

        win.init();
        win.setup_actions();

        // Set icons for shell
        gtk::Window::set_default_icon_name(config::APP_ID);

        win
    }

    fn get_private(&self) -> &imp::SolanumWindow {
        &imp::SolanumWindow::from_instance(self)
    }

    fn application(&self) -> SolanumApplication {
        gtk::traits::GtkWindowExt::application(self)
            .unwrap()
            .downcast::<SolanumApplication>()
            .unwrap()
    }

    fn init(&self) {
        let imp = self.get_private();
        let timer_label = &*imp.timer_label;
        let app = self.application();
        let settings = app.gsettings();

        if config::APP_ID.ends_with("Devel") {
            self.add_css_class("devel");
        }

        self.update_lap_label();

        let min = settings.get::<u32>("lap-length");
        imp.timer.set_duration(min);
        timer_label.set_label(&format!("{:>02}∶00", min));

        imp.timer.connect_countdown_update(
            clone!(@weak self as win => move |_, minutes, seconds| {
                win.update_countdown(minutes, seconds);
            }),
        );

        imp.timer
            .connect_lap(clone!(@weak self as win => move |_, lap_type| {
                win.next_lap(lap_type);
            }));
    }

    // Set up actions on the Window itself
    fn setup_actions(&self) {
        action!(
            self,
            "menu",
            clone!(@weak self as win => move |_, _| {
                let imp = win.get_private();
                let menu_button = &*imp.menu_button;
                menu_button.popover().unwrap().popup();
            })
        );

        // Stateful actions allow us to set a state each time an action is activated
        let timer_on = false;
        stateful_action!(
            self,
            "toggle-timer",
            timer_on,
            clone!(@weak self as win => move |a, v| {
                win.on_timer_toggled(a, v)
            })
        );

        action!(
            self,
            "skip",
            clone!(@weak self as win => move |_, _| {
                win.skip();
            })
        );
    }

    fn skip(&self) {
        let imp = self.get_private();
        let lap_type = imp.timer.lap_type();

        let next_lap = if lap_type == LapType::Pomodoro {
            LapType::Break
        } else {
            LapType::Pomodoro
        };

        self.update_lap(next_lap);
        if !self.is_active() {
            self.present();
        }
    }

    fn update_countdown(&self, min: u32, sec: u32) -> glib::Continue {
        let imp = self.get_private();
        let label = &*imp.timer_label;
        label.set_label(&format!("{:>02}∶{:>02}", min, sec));
        glib::Continue(true)
    }

    fn update_lap(&self, lap_type: LapType) {
        let imp = self.get_private();
        let label = &*imp.lap_label;
        let timer = &imp.timer;
        let app = self.application();
        let settings = app.gsettings();

        timer.set_lap_type(lap_type);

        let lap_number = &imp.pomodoro_count;
        println!("Setting lap to {:?}", lap_type);

        match lap_type {
            LapType::Pomodoro => {
                let length = settings.get::<u32>("lap-length");
                self.update_lap_label();
                timer.set_duration(length);
                self.set_timer_label_from_secs(length * 60);
            }
            LapType::Break => {
                if lap_number.get() >= settings.get::<u32>("sessions-until-long-break") {
                    let length = settings.get::<u32>("long-break-length");
                    lap_number.set(1);
                    label.set_label(&i18n("Long Break"));
                    timer.set_duration(length);
                    self.set_timer_label_from_secs(length * 60);
                } else {
                    let length = settings.get::<u32>("short-break-length");
                    lap_number.set(lap_number.get() + 1);
                    label.set_label(&i18n("Short Break"));
                    timer.set_duration(length);
                    self.set_timer_label_from_secs(length * 60);
                }
            }
        };
    }

    // Callback to run whenever the timer is toggled - by button or action
    fn on_timer_toggled(&self, action: &gio::SimpleAction, _variant: Option<&glib::Variant>) {
        let imp = self.get_private();
        let action_state: bool = action.state().unwrap().get().unwrap();
        let timer_on = !action_state;
        action.set_state(&timer_on.to_variant());

        let skip = self
            .lookup_action("skip")
            .unwrap()
            .downcast::<gio::SimpleAction>()
            .unwrap();
        skip.set_enabled(!timer_on);

        let timer = &imp.timer;
        let timer_label = &*imp.timer_label;
        let timer_button = &*imp.timer_button;

        if timer_on {
            timer.start();
            self.play_sound(BEEP_URI);
            timer_button.set_icon_name("media-playback-pause-symbolic");
            timer_label.remove_css_class("blinking");
            timer_button.remove_css_class("suggested-action");
        } else {
            timer.stop();
            timer_button.set_icon_name("media-playback-start-symbolic");
            timer_label.add_css_class("blinking");
            timer_button.add_css_class("suggested-action");
        }
    }

    // Util for setting the timer label when given seconds
    fn set_timer_label_from_secs(&self, secs: u32) {
        let imp = self.get_private();
        let label = &*imp.timer_label;
        let min = secs / 60;
        let secs = secs % 60;
        label.set_label(&format!("{:>02}∶{:>02}", min, secs));
    }

    fn play_sound(&self, uri: &str) {
        let player = &self.get_private().player;
        player.set_uri(uri);
        player.play();
    }

    fn send_notifcation(&self, lap_type: LapType) {
        if !self.is_active() {
            let notif = gio::Notification::new(&i18n("Solanum"));
            // Set notification text based on lap type
            let (title, body, button) = match lap_type {
                LapType::Pomodoro => (
                    i18n("Back to Work"),
                    i18n("Ready to keep working?"),
                    i18n("Start Working"),
                ),
                LapType::Break => (
                    i18n("Break Time"),
                    i18n("Stretch your legs, and drink some water."),
                    i18n("Start Break"),
                ),
            };
            notif.set_title(&title);
            notif.set_body(Some(&body));
            notif.add_button(&button, "app.toggle-timer");
            notif.add_button(&i18n("Skip"), "app.skip");
            let app = self.application();
            app.send_notification(Some("timer-notif"), &notif);
        }
        self.play_sound(CHIME_URI);
    }

    fn update_lap_label(&self) {
        let imp = self.get_private();

        // Translators: Every pomodoro session can range from 1-99 laps,
        // so {} will contain a number between 1 and 99. Lap is always singular.
        imp.lap_label.set_label(&ni18n_f(
            "Lap {}",
            "Lap {}",
            imp.pomodoro_count.get(),
            &[&imp.pomodoro_count.get().to_string()],
        ));
    }

    // Pause the timer and move to the next lap
    fn next_lap(&self, lap_type: LapType) -> glib::Continue {
        // This stops the timer and sets the styling we need
        let action = self.lookup_action("toggle-timer").unwrap();
        action.activate(None);

        self.update_lap(lap_type);
        self.send_notifcation(lap_type);
        glib::Continue(true)
    }
}
