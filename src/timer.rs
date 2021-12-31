// timer.rs
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

use glib::prelude::*;
use glib::subclass::{prelude::*, Signal};
use glib::{clone, GEnum, StaticType};

// `Rc`s are Reference Counters. They allow us to clone objects,
// while actually referencing at different places.
// A `RefCell` allows for interior mutablility.
use std::cell::Cell;
use std::time::{Duration, Instant};

// `Lazy` is a structure for Lazy loading things during runtime.
use once_cell::sync::Lazy;

#[derive(Copy, Clone, Debug, Eq, PartialEq, GEnum)]
#[genum(type_name = "SolanumLapType")]
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

    #[derive(Debug, Default)]
    pub struct Timer {
        pub running: Cell<bool>,
        pub instant: Cell<Option<Instant>>,
        pub duration: Cell<Duration>,
        pub lap_type: Cell<LapType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Timer {
        const NAME: &'static str = "SolanumTimer";
        type Type = super::Timer;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Timer {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder(
                        "countdown-update",
                        &[u32::static_type().into(), u32::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .build(),
                    Signal::builder("lap", &[], <()>::static_type().into()).build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
}

glib::wrapper! {
    pub struct Timer(ObjectSubclass<imp::Timer>);
}

impl Timer {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to initialize Timer object")
    }

    pub fn connect_countdown_update<F: Fn(&Self, u32, u32) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("countdown-update", true, move |values| {
            let timer = values[0].get::<Self>().unwrap();
            let minutes = values[1].get::<u32>().unwrap();
            let seconds = values[2].get::<u32>().unwrap();

            f(&timer, minutes, seconds);

            None
        })
        .unwrap()
    }

    pub fn connect_lap<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("lap", true, move |values| {
            let timer = values[0].get::<Self>().unwrap();

            f(&timer);

            None
        })
        .unwrap()
    }

    fn get_private(&self) -> &imp::Timer {
        &imp::Timer::from_instance(self)
    }

    // Pass the duration in minutes
    pub fn set_duration(&self, duration: u32) {
        let imp = self.get_private();

        imp.instant.set(Some(Instant::now()));
        imp.duration.set(Duration::new((duration * 60).into(), 0));
    }

    pub fn start(&self) {
        let imp = self.get_private();

        imp.running.set(true);
        imp.instant.set(Some(Instant::now()));

        // Every 100 milliseconds, this closure gets called in order to update the timer
        glib::timeout_add_local(
            std::time::Duration::from_millis(100),
            clone!(@weak self as timer => @default-return glib::Continue(false), move || {
                let imp = timer.get_private();
                if imp.running.get() {
                    let instant = imp.instant.get().expect("Timer is running, but no instant is set.");
                    let duration = imp.duration.get();
                    if let Some(difference) = duration.checked_sub(instant.elapsed()) {
                        let (minutes, seconds) = duration_to_mins_and_secs(difference);
                        timer.emit_by_name("countdown-update", &[&minutes, &seconds]).expect("Could not emit \"countdown-update\" signal on SolanumTimer");
                        return glib::Continue(true);
                    } else {
                        let new_lt = {
                            if imp.lap_type.get() == LapType::Pomodoro {
                                LapType::Break
                            } else {
                                LapType::Pomodoro
                            }
                        };
                        timer.set_lap_type(new_lt);
                        timer.emit_by_name("lap", &[]).expect("Could not emit \"lap\" signal on SolanumTimer");
                        return glib::Continue(false);
                    }
                }

                glib::Continue(false)
            }),
        );
    }

    pub fn stop(&self) {
        let imp = self.get_private();

        imp.running.set(false);

        // When paused, set the timer so that it will resume where the user left off
        let elapsed = imp.instant.get().unwrap().elapsed();
        if let Some(difference) = imp.duration.get().checked_sub(elapsed) {
            imp.duration.set(difference);
        }

        println!("Timer stopped!")
    }

    pub fn set_lap_type(&self, new_type: LapType) {
        let imp = self.get_private();
        imp.lap_type.set(new_type);
    }

    pub fn lap_type(&self) -> LapType {
        let imp = self.get_private();
        imp.lap_type.get()
    }

    pub fn running(&self) -> bool {
        self.get_private().running.get()
    }
}

fn duration_to_mins_and_secs(duration: Duration) -> (u32, u32) {
    use std::convert::TryInto;

    let mut seconds = duration.as_secs();
    let minutes = seconds / 60;
    seconds %= 60;

    let minutes = minutes.try_into().unwrap();
    let seconds = seconds.try_into().unwrap();

    (minutes, seconds)
}
