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
use libhandy::subclass::prelude as hdy;

use once_cell::unsync::OnceCell;

use std::cell::Cell;

use crate::config;
use crate::i18n::*;
use crate::timer::{LapType, Timer, TimerActions};

static POMODORO_SECONDS: u64 = 1500; // == 25 Minutes
static SHORT_BREAK_SECONDS: u64 = 300; // == 5 minutes
static LONG_BREAK_SECONDS: u64 = 900; // == 15 minutes
static POMODOROS_UNTIL_LONG_BREAK: u32 = 4;

static CHIME_URI: &str = "resource:///org/gnome/Solanum/chime.ogg";
static BEEP_URI: &str = "resource:///org/gnome/Solanum/beep.ogg";
static WINDOW_URI: &str = "/org/gnome/Solanum/window.ui";

#[derive(Debug, CompositeTemplate)]
pub struct SolanumWindowPriv {
    pomodoro_count: Cell<u32>,
    timer: OnceCell<Timer>,
    lap_type: Cell<LapType>,
    player: gstreamer_player::Player,
    #[template_child]
    lap_label: TemplateChild<gtk::Label>,
    #[template_child]
    timer_label: TemplateChild<gtk::Label>,
    #[template_child]
    timer_button: TemplateChild<gtk::Button>,
    #[template_child]
    menu_button: TemplateChild<gtk::MenuButton>,
}

impl ObjectSubclass for SolanumWindowPriv {
    const NAME: &'static str = "SolanumWindow";
    type Type = SolanumWindow;
    type ParentType = libhandy::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib::object_subclass!();

    fn new() -> Self {
        Self {
            pomodoro_count: Cell::new(1),
            timer: OnceCell::new(),
            lap_type: Cell::new(LapType::Pomodoro),
            player: gstreamer_player::Player::new(None, None),
            lap_label: TemplateChild::default(),
            timer_label: TemplateChild::default(),
            timer_button: TemplateChild::default(),
            menu_button: TemplateChild::default(),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_template_from_resource(WINDOW_URI);
        Self::bind_template_children(klass);
    }

    fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
        obj.init_template();
    }
}

impl ObjectImpl for SolanumWindowPriv {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        let builder = gtk::Builder::from_resource("/org/gnome/Solanum/gtk/help-overlay.ui");
        let help_overlay = builder.get_object("help_overlay").unwrap();
        obj.set_help_overlay(Some(&help_overlay));
    }
}

// We don't need to override any vfuncs here, but since they're superclasses
// we need to declare the blank impls
impl WidgetImpl for SolanumWindowPriv {}
impl WindowImpl for SolanumWindowPriv {}
impl ApplicationWindowImpl for SolanumWindowPriv {}
impl hdy::ApplicationWindowImpl for SolanumWindowPriv {}

glib::wrapper! {
    pub struct SolanumWindow(ObjectSubclass<SolanumWindowPriv>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libhandy::ApplicationWindow,
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

    fn get_private(&self) -> &SolanumWindowPriv {
        &SolanumWindowPriv::from_instance(self)
    }

    fn init(&self) {
        let priv_ = self.get_private();
        let timer_label = priv_.timer_label.get();

        self.update_lap_label();

        let min = POMODORO_SECONDS / 60;
        let secs = POMODORO_SECONDS % 60;
        timer_label.set_label(&format!("{:>02}∶{:>02}", min, secs));

        // Set up (Sender, Receiver) of actions for the timer
        let (tx, rx) = glib::MainContext::channel::<TimerActions>(glib::PRIORITY_DEFAULT);
        priv_
            .timer
            .set(Timer::new(POMODORO_SECONDS, tx))
            .expect("Could not initialize timer");
        // The receiver will get certain actions from the Timer and run operations on the Window
        rx.attach(
            None,
            clone!(@weak self as win => @default-return glib::Continue(true), move |action| match action {
                TimerActions::CountdownUpdate(min, sec) => win.update_countdown(min, sec),
                TimerActions::Lap(lap_type) => win.next_lap(lap_type),
            }),
        );
    }

    // Set up actions on the Window itself
    fn setup_actions(&self) {
        action!(
            self,
            "menu",
            clone!(@weak self as win => move |_, _| {
                let priv_ = win.get_private();
                let menu_button = priv_.menu_button.get();
                menu_button.get_popover().unwrap().popup();
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
        let priv_ = self.get_private();
        let lap_type = priv_.lap_type.get();

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
        let priv_ = self.get_private();
        let label = &*priv_.timer_label;
        label.set_label(&format!("{:>02}∶{:>02}", min, sec));
        glib::Continue(true)
    }

    fn update_lap(&self, lap_type: LapType) {
        let priv_ = self.get_private();
        let label = &*priv_.lap_label;
        let timer = priv_.timer.get().unwrap();

        priv_.lap_type.set(lap_type);
        timer.set_lap_type(lap_type);

        let lap_number = &priv_.pomodoro_count;
        println!("Setting lap to {:?}", lap_type);

        match lap_type {
            LapType::Pomodoro => {
                self.update_lap_label();
                timer.set_duration(POMODORO_SECONDS);
                self.set_timer_label_from_secs(POMODORO_SECONDS);
            }
            LapType::Break => {
                if lap_number.get() >= POMODOROS_UNTIL_LONG_BREAK {
                    lap_number.set(1);
                    label.set_label(&i18n("Long Break"));
                    timer.set_duration(LONG_BREAK_SECONDS);
                    self.set_timer_label_from_secs(LONG_BREAK_SECONDS);
                } else {
                    lap_number.set(lap_number.get() + 1);
                    label.set_label(&i18n("Short Break"));
                    timer.set_duration(SHORT_BREAK_SECONDS);
                    self.set_timer_label_from_secs(SHORT_BREAK_SECONDS);
                }
            }
        };
    }

    // Callback to run whenever the timer is toggled - by button or action
    fn on_timer_toggled(&self, action: &gio::SimpleAction, _variant: Option<&glib::Variant>) {
        let priv_ = self.get_private();
        let action_state: bool = action.get_state().unwrap().get().unwrap();
        let timer_on = !action_state;
        action.set_state(&timer_on.to_variant());

        let skip = self
            .lookup_action("skip")
            .unwrap()
            .downcast::<gio::SimpleAction>()
            .unwrap();
        skip.set_enabled(!timer_on);

        let timer = self.get_private().timer.get().unwrap();
        let timer_label = &*priv_.timer_label;
        let timer_button = &*priv_.timer_button;

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

    // Util for initializing the timer based on the contants at the top
    fn set_timer_label_from_secs(&self, secs: u64) {
        let priv_ = self.get_private();
        let label = &*priv_.timer_label;
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
            let app = self.get_application().unwrap();
            app.send_notification(Some("timer-notif"), &notif);
        }
        self.play_sound(CHIME_URI);
    }

    fn update_lap_label(&self) {
        let priv_ = self.get_private();

        // Translators: Every pomodoro session is made of 4 laps,
        // so {} will contain a number between 1 and 4. Lap is always singular.
        priv_.lap_label.set_label(&ni18n_f(
            "Lap {}",
            "Lap {}",
            priv_.pomodoro_count.get(),
            &[&priv_.pomodoro_count.get().to_string()],
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
