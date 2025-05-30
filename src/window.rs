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

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use glib::{clone, Enum};
use gtk::CompositeTemplate;

use glib::subclass;
use glib::subclass::prelude::*;
use gtk::prelude::IsA;
use gtk::subclass::prelude::*;
use libadwaita::subclass::prelude::*;

use std::cell::Cell;

use crate::app::SolanumApplication;
use crate::config;
use crate::i18n::*;
use crate::timer::Timer;

static CHIME_URI: &str = "resource:///org/gnome/Solanum/chime.ogg";
static BEEP_URI: &str = "resource:///org/gnome/Solanum/beep.ogg";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Enum)]
#[enum_type(name = "SolanumLapType")]
pub enum LapType {
    Pomodoro,
    Break,
}

impl Default for LapType {
    fn default() -> Self {
        Self::Pomodoro
    }
}

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/Solanum/window.ui")]
    pub struct SolanumWindow {
        pub pomodoro_count: Cell<u32>,
        pub timer: Timer,
        pub player: gstreamer_play::Play,
        pub lap_type: Cell<LapType>,

        #[template_child]
        pub lap_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub timer_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub timer_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub large_text_bp: TemplateChild<libadwaita::Breakpoint>,
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
                player: gstreamer_play::Play::new(None::<gstreamer_play::PlayVideoRenderer>),
                lap_type: Default::default(),
                lap_label: TemplateChild::default(),
                timer_label: TemplateChild::default(),
                timer_button: TemplateChild::default(),
                menu_button: TemplateChild::default(),
                large_text_bp: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.toggle-timer", None, move |win, _, _| {
                win.toggle_timer();
            });

            klass.install_action("win.reset", None, move |win, _, _| {
                win.reset();
            });

            klass.install_action("win.skip", None, move |win, _, _| {
                win.next_lap(false);

                if !win.is_active() {
                    win.present();
                }
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SolanumWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let timer_label = &*self.timer_label;
            let lap_label = &*self.lap_label;
            self.large_text_bp.connect_apply(clone!(
                #[weak]
                timer_label,
                #[weak]
                lap_label,
                move |_| {
                    timer_label.add_css_class("large-timer");
                    lap_label.remove_css_class("heading");
                    lap_label.add_css_class("title-4");
                }
            ));

            self.large_text_bp.connect_unapply(clone!(
                #[weak]
                timer_label,
                #[weak]
                lap_label,
                move |_| {
                    timer_label.remove_css_class("large-timer");
                    lap_label.remove_css_class("title-4");
                    lap_label.add_css_class("heading");
                }
            ));
        }
    }

    // We don't need to override any vfuncs here, but since they're superclasses
    // we need to declare the blank impls
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
        let win = glib::Object::builder::<Self>()
            .property("application", app)
            .build();

        win.init();

        // Set icons for shell
        gtk::Window::set_default_icon_name(config::APP_ID);

        win
    }

    fn application(&self) -> SolanumApplication {
        gtk::prelude::GtkWindowExt::application(self)
            .unwrap()
            .downcast::<SolanumApplication>()
            .unwrap()
    }

    fn init(&self) {
        let imp = self.imp();
        let timer_label = &*imp.timer_label;
        let app = self.application();
        let settings = app.gsettings();

        timer_label.set_direction(gtk::TextDirection::Ltr);

        if config::APP_ID.ends_with("Devel") {
            self.add_css_class("devel");
        }

        self.update_lap_label();

        let min = settings.uint("lap-length");
        imp.timer.set_duration(min);
        timer_label.set_label(&format!("{:>02}∶00", min));

        imp.timer.connect_countdown_update(clone!(
            #[weak(rename_to = win)]
            self,
            move |_, minutes, seconds| {
                win.update_countdown(minutes, seconds);
            }
        ));

        imp.timer.connect_lap(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                win.toggle_timer();
                win.next_lap(true);
            }
        ));
    }

    fn update_countdown(&self, min: u32, sec: u32) -> glib::ControlFlow {
        let imp = self.imp();
        let label = &*imp.timer_label;
        label.set_label(&format!("{:>02}∶{:>02}", min, sec));
        glib::ControlFlow::Continue
    }

    fn update_lap(&self, lap_type: LapType) {
        let imp = self.imp();
        let label = &*imp.lap_label;
        let timer = &imp.timer;
        let app = self.application();
        let settings = app.gsettings();

        imp.lap_type.set(lap_type);

        let lap_number = &imp.pomodoro_count;
        println!("Setting lap to {:?}", lap_type);

        match lap_type {
            LapType::Pomodoro => {
                let length = settings.get("lap-length");
                self.update_lap_label();
                timer.set_duration(length);
                self.set_timer_label_from_secs(length * 60);
            }
            LapType::Break => {
                if lap_number.get() >= settings.uint("sessions-until-long-break") {
                    let length = settings.uint("long-break-length");
                    lap_number.set(1);
                    label.set_label(&i18n("Long Break"));
                    timer.set_duration(length);
                    self.set_timer_label_from_secs(length * 60);
                } else {
                    let length = settings.uint("short-break-length");
                    lap_number.set(lap_number.get() + 1);
                    label.set_label(&i18n("Short Break"));
                    timer.set_duration(length);
                    self.set_timer_label_from_secs(length * 60);
                }
            }
        };
    }

    // Callback to run whenever the timer is toggled - by button or action
    fn toggle_timer(&self) {
        let imp = self.imp();
        let app = self.application();
        let settings = app.gsettings();
        let fullscreen = settings.boolean("fullscreen-break");

        let start_timer = !imp.timer.running();
        self.action_set_enabled("win.skip", !start_timer);

        if start_timer {
            let app = self.application();
            app.withdraw_notification("timer-notif");
            imp.timer.start();
            self.play_sound(BEEP_URI);
            imp.timer_button
                .set_icon_name("media-playback-pause-symbolic");
            imp.timer_label.remove_css_class("blinking");
            imp.timer_button.remove_css_class("suggested-action");
            if fullscreen {
                if imp.lap_type.get() == LapType::Break {
                    self.fullscreen();
                } else {
                    self.unfullscreen();
                }
            }
        } else {
            imp.timer.stop();
            imp.timer_button
                .set_icon_name("media-playback-start-symbolic");
            imp.timer_label.add_css_class("blinking");
            imp.timer_button.add_css_class("suggested-action");
        }

        // !start_timer = only allow restarting when the timer is paused
        self.action_set_enabled("win.reset", !start_timer)
    }

    // Callback for resetting the application to the initial state.
    fn reset(&self) {
        println!("Resetting to the initial state");
        let imp = self.imp();

        // Reset the user interface to the stopped state.
        imp.timer.stop();
        imp.timer_button
            .set_icon_name("media-playback-start-symbolic");
        imp.timer_label.add_css_class("blinking");
        imp.timer_button.add_css_class("suggested-action");

        imp.pomodoro_count.set(1);
        self.update_lap(LapType::Pomodoro);
    }

    // Util for setting the timer label when given seconds
    fn set_timer_label_from_secs(&self, secs: u32) {
        let imp = self.imp();
        let label = &*imp.timer_label;
        let min = secs / 60;
        let secs = secs % 60;
        label.set_label(&format!("{:>02}∶{:>02}", min, secs));
    }

    fn play_sound(&self, uri: &str) {
        let player = &self.imp().player;
        player.set_uri(Some(uri));
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
            notif.set_priority(gio::NotificationPriority::Urgent);
            notif.add_button(&button, "app.toggle-timer");
            notif.add_button(&i18n("Skip"), "app.skip");
            let app = self.application();
            app.send_notification(Some("timer-notif"), &notif);
        }
        self.play_sound(CHIME_URI);
    }

    fn update_lap_label(&self) {
        let imp = self.imp();

        // Translators: Every pomodoro session can range from 1-99 laps,
        // so {} will contain a number between 1 and 99. Lap is always singular.
        imp.lap_label.set_label(&ni18n_f(
            "Lap {}",
            "Lap {}",
            imp.pomodoro_count.get(),
            &[&imp.pomodoro_count.get().to_string()],
        ));
    }

    // Move to the next lap
    fn next_lap(&self, notify: bool) {
        let imp = self.imp();
        let lap_type = imp.lap_type.get();

        let next_lap = if lap_type == LapType::Pomodoro {
            LapType::Break
        } else {
            LapType::Pomodoro
        };

        self.update_lap(next_lap);

        if notify {
            self.send_notifcation(next_lap);
        }
    }
}
