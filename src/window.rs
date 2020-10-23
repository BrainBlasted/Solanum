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

use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use gtk::prelude::IsA;
use gtk::subclass::prelude::*;

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

#[derive(Clone, Debug)]
struct Widgets {
    handle: gtk::WindowHandle,
    menu_button: gtk::MenuButton,
    lap_label: gtk::Label,
    timer_label: gtk::Label,
    timer_button: gtk::Button,
    skip_button: gtk::Button,
}

#[derive(Debug)]
pub struct SolanumWindowPriv {
    widgets: OnceCell<Widgets>,
    pomodoro_count: Cell<u32>,
    timer: OnceCell<Timer>,
    lap_type: Cell<LapType>,
    player: gstreamer_player::Player,
}

impl ObjectSubclass for SolanumWindowPriv {
    const NAME: &'static str = "SolanumWindow";
    type ParentType = gtk::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            widgets: OnceCell::new(),
            pomodoro_count: Cell::new(1),
            timer: OnceCell::new(),
            lap_type: Cell::new(LapType::Pomodoro),
            player: gstreamer_player::Player::new(None, None),
        }
    }
}

impl ObjectImpl for SolanumWindowPriv {
    // After the widget is constructed we want to initialize & add it's children
    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);

        let builder = gtk::Builder::from_resource("/org/gnome/Solanum/menu.ui");

        let count = self.pomodoro_count.clone().into_inner();

        let dummy_titlebar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        dummy_titlebar.set_visible(false);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox.set_vexpand(true);
        vbox.set_valign(gtk::Align::Center);
        vbox.set_margin_top(72);
        add_style_class!(vbox, @main_box);

        let titlebox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let start_control = gtk::WindowControlsBuilder::new()
            .side(gtk::PackType::Start)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .margin_top(6)
            .margin_start(6)
            .margin_bottom(6)
            .build();
        let end_control = gtk::WindowControlsBuilder::new()
            .side(gtk::PackType::End)
            .halign(gtk::Align::End)
            .hexpand(true)
            .margin_top(6)
            .margin_end(6)
            .margin_bottom(6)
            .build();
        titlebox.prepend(&start_control);
        titlebox.append(&end_control);

        let vbox2 = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox2.append(&titlebox);
        vbox2.append(&vbox);

        let lap_label = gtk::Label::new(Some(&i18n_f("Lap {}", &[&count.to_string()])));
        add_style_class!(lap_label, @lap_label);

        let timer_label = gtk::Label::new(None);
        let attrs = pango::AttrList::new();
        let tnum = pango::Attribute::new_font_features("tnum=1").unwrap();
        attrs.insert(tnum);
        timer_label.set_attributes(Some(&attrs));
        add_style_class!(timer_label, &["blinking", "timer_label"]);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let timer_button = gtk::Button::from_icon_name(Some("media-playback-start-symbolic"));
        timer_button.set_action_name(Some("win.toggle-timer"));
        add_style_class!(
            timer_button,
            &[
                "pill_button",
                "large_button",
                "suggested-action",
                "timer_button"
            ]
        );

        let skip_button = gtk::Button::from_icon_name(Some("media-seek-forward-symbolic"));
        skip_button.set_action_name(Some("win.skip"));
        add_style_class!(
            skip_button,
            &["pill_button", "large_button", "timer_button"]
        );

        hbox.append(&timer_button);
        hbox.append(&skip_button);
        hbox.set_halign(gtk::Align::Center);

        vbox.append(&lap_label);
        vbox.append(&timer_label);
        vbox.append(&hbox);

        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        let app_menu = get_widget!(builder, gio::MenuModel, @app_menu);
        menu_button.set_menu_model(Some(&app_menu));
        menu_button.set_margin_start(24);
        menu_button.set_margin_end(24);
        menu_button.set_margin_top(24);
        menu_button.set_margin_bottom(24);
        menu_button.set_halign(gtk::Align::End);
        add_style_class!(menu_button, @pill_button);
        vbox2.append(&menu_button);

        vbox2.set_property_width_request(360);

        let handle = gtk::WindowHandle::new();
        handle.set_child(Some(&vbox2));

        let window = obj.downcast_ref::<gtk::ApplicationWindow>().unwrap();
        window.set_titlebar(Some(&dummy_titlebar));
        window.set_child(Some(&handle));
        window.set_default_size(600, 300);
        remove_style_class!(window, &["solid-csd"]);

        let w = window.downcast_ref::<SolanumWindow>().unwrap();
        w.setup_actions();

        // Set up (Sender, Receiver) of actions for the timer
        let (tx, rx) = glib::MainContext::channel::<TimerActions>(glib::PRIORITY_DEFAULT);
        self.timer
            .set(Timer::new(POMODORO_SECONDS, tx))
            .expect("Could not initialize timer");
        let min = POMODORO_SECONDS / 60;
        let secs = POMODORO_SECONDS % 60;
        timer_label.set_label(&format!("{:>02}∶{:>02}", min, secs));
        // The receiver will get certain actions from the Timer and run operations on the Window
        rx.attach(
            None,
            clone!(@weak w => @default-return glib::Continue(true), move |action| match action {
                TimerActions::CountdownUpdate(min, sec) => w.update_countdown(min, sec),
                TimerActions::Lap(lap_type) => w.next_lap(lap_type),
            }),
        );

        self.widgets
            .set(Widgets {
                handle,
                menu_button,
                lap_label,
                timer_label,
                timer_button,
                skip_button,
            })
            .expect("Could not set widget state for main window");

        // Set icons for shell
        gtk::Window::set_default_icon_name(config::APP_ID);
    }
}

// We don't need to override any vfuncs here, but since they're superclasses
// we need to declare the blank impls
impl WidgetImpl for SolanumWindowPriv {}
impl WindowImpl for SolanumWindowPriv {}
impl ApplicationWindowImpl for SolanumWindowPriv {}

glib_wrapper! {
    pub struct SolanumWindow(
        Object<subclass::simple::InstanceStruct<SolanumWindowPriv>,
        subclass::simple::ClassStruct<SolanumWindowPriv>,
        SolanumWindowClass>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;

    match fn {
        get_type => || SolanumWindowPriv::get_type().to_glib(),
    }
}

impl SolanumWindow {
    pub fn new<P: IsA<gtk::Application> + glib::value::ToValue>(app: &P) -> Self {
        glib::Object::new(Self::static_type(), &[("application", app)])
            .expect("Failed to create SolanumWindow")
            .downcast::<SolanumWindow>()
            .expect("Created SolanumWindow is of wrong type")
    }

    fn get_private(&self) -> &SolanumWindowPriv {
        &SolanumWindowPriv::from_instance(self)
    }

    fn get_widgets(&self) -> Widgets {
        let priv_ = self.get_private();
        priv_.widgets.clone().into_inner().unwrap()
    }

    // Set up actions on the Window itself
    fn setup_actions(&self) {
        action!(
            self,
            "menu",
            clone!(@weak self as win => move |_, _| {
                let widgets = win.get_widgets();
                widgets.menu_button.get_popover().unwrap().popup();
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
        let label = self.get_widgets().lap_label;
        let lap_type = priv_.lap_type.get();
        let lap_number = &priv_.pomodoro_count;
        let timer = priv_.timer.get().unwrap();

        let next_lap = if lap_type == LapType::Pomodoro {
            LapType::Break
        } else {
            LapType::Pomodoro
        };

        priv_.lap_type.replace(next_lap);

        match next_lap {
            LapType::Pomodoro => {
                label.set_label(&i18n_f("Lap {}", &[&lap_number.get().to_string()]));
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

        if !self.is_active() {
            self.present();
        }
    }

    fn update_countdown(&self, min: u32, sec: u32) -> glib::Continue {
        let widgets = self.get_widgets();
        let label = widgets.timer_label;
        label.set_label(&format!("{:>02}∶{:>02}", min, sec));
        glib::Continue(true)
    }

    // Callback to run whenever the timer is toggled - by button or action
    fn on_timer_toggled(&self, action: &gio::SimpleAction, _variant: Option<&glib::Variant>) {
        let action_state: bool = action.get_state().unwrap().get().unwrap();
        let timer_on = !action_state;
        action.set_state(&timer_on.to_variant());

        let skip = self.lookup_action("skip").unwrap();

        let widgets = self.get_widgets();
        let timer_image = widgets
            .timer_button
            .get_child()
            .unwrap()
            .downcast::<gtk::Image>()
            .unwrap();
        let timer = self.get_private().timer.get().unwrap();

        if timer_on {
            timer.start();
            self.play_sound(BEEP_URI);
            timer_image.set_from_icon_name(Some("media-playback-pause-symbolic"));
            add_style_class!(widgets.timer_label, @blue_text);
            remove_style_class!(widgets.timer_label, @blinking);
            remove_style_class!(widgets.timer_button, &["suggested-action"]);
            let _ = skip.set_property("enabled", &false);
        } else {
            timer.stop();
            timer_image.set_from_icon_name(Some("media-playback-start-symbolic"));
            add_style_class!(widgets.timer_label, @blinking);
            remove_style_class!(widgets.timer_label, @blue_text);
            add_style_class!(widgets.timer_button, &["suggested-action"]);
            let _ = skip.set_property("enabled", &true);
        }
    }

    // Util for initializing the timer based on the contants at the top
    fn set_timer_label_from_secs(&self, secs: u64) {
        let label = self.get_widgets().timer_label;
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

    // Pause the timer and move to the next lap
    fn next_lap(&self, lap_type: LapType) -> glib::Continue {
        let priv_ = self.get_private();
        let label = self.get_widgets().lap_label;
        let timer = priv_.timer.get().unwrap();
        priv_.lap_type.set(lap_type);

        // This stops the timer and sets the styling we need
        let action = self.lookup_action("toggle-timer").unwrap();
        action.activate(None);

        let lap_number = &priv_.pomodoro_count;
        println!("Lapping with {:?}", lap_type);

        match lap_type {
            LapType::Pomodoro => {
                label.set_label(&i18n_f("Lap {}", &[&lap_number.get().to_string()]));
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
        self.send_notifcation(lap_type);
        glib::Continue(true)
    }
}
