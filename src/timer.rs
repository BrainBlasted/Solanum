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
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimerState {
    Running,
    Stopped,
}

impl Default for TimerState {
    fn default() -> Self {
        TimerState::Stopped
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Timer {
        pub state: Rc<RefCell<TimerState>>,
        pub instant: Rc<RefCell<Option<Instant>>>,
        pub duration: Rc<RefCell<Duration>>,
        pub lap_type: Rc<RefCell<LapType>>,
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
                    Signal::builder(
                        "lap",
                        &[LapType::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .build(),
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
    pub fn new(duration: u64) -> Self {
        let obj: Self = glib::Object::new::<Self>(&[]).expect("Failed to initialize Timer object");
        obj.set_duration(duration);
        obj
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

    pub fn connect_lap<F: Fn(&Self, LapType) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("lap", true, move |values| {
            let timer = values[0].get::<Self>().unwrap();
            let lap_type = values[1].get::<LapType>().unwrap();

            f(&timer, lap_type);

            None
        })
        .unwrap()
    }

    fn get_private(&self) -> &imp::Timer {
        &imp::Timer::from_instance(self)
    }

    pub fn set_duration(&self, duration: u64) {
        let imp = self.get_private();

        let mut i = imp.instant.borrow_mut();
        *i = Some(Instant::now());
        let mut d = imp.duration.borrow_mut();
        *d = Duration::new(duration, 0);
    }

    pub fn start(&self) {
        let imp = self.get_private();

        let mut state = imp.state.borrow_mut();
        *state = TimerState::Running;
        let mut instant = imp.instant.borrow_mut();
        *instant = Some(Instant::now());

        let s = &imp.state;
        let i = &imp.instant;
        let d = &imp.duration;
        let lt = &imp.lap_type;
        // Every 100 milliseconds, this closure gets called in order to update the timer
        glib::timeout_add_local(
            std::time::Duration::from_millis(100),
            clone!(@weak self as timer, @weak s, @weak i, @weak d, @weak lt => @default-return glib::Continue(false), move || {
                if *s.borrow() == TimerState::Running {
                    if let Some(difference) = {
                        let instant = i.borrow().expect("Timer is running, but no instant is set.");
                        let duration = d.borrow();
                        duration.checked_sub(instant.elapsed())
                    } {
                        let msm = duration_to_ms(difference);
                        let _ = timer.emit_by_name("countdown-update", &[&msm.0, &msm.1]);
                        return glib::Continue(true);
                    } else {
                        let new_lt = {
                            if *lt.borrow() == LapType::Pomodoro {
                                LapType::Break
                            } else {
                                LapType::Pomodoro
                            }
                        };
                        timer.set_lap_type(new_lt);
                        let _ = timer.emit_by_name("lap", &[&new_lt]);
                        return glib::Continue(false);
                    }
                }

                glib::Continue(false)
            }),
        );
    }

    pub fn stop(&self) {
        let imp = self.get_private();

        let mut state = imp.state.borrow_mut();
        *state = TimerState::Stopped;

        // When paused, set the timer so that it will resume where the user left off
        let mut duration = imp.duration.borrow_mut();
        let instant = imp.instant.borrow().unwrap();
        let elapsed = instant.elapsed();
        if let Some(difference) = duration.checked_sub(elapsed) {
            *duration = difference;
        }

        println!("Timer stopped!")
    }

    pub fn set_lap_type(&self, new_type: LapType) {
        let imp = self.get_private();
        let mut lap_type = imp.lap_type.borrow_mut();

        *lap_type = new_type;
    }
}

fn duration_to_ms(duration: Duration) -> (u32, u32) {
    use std::convert::TryInto;

    let mut seconds = duration.as_secs();
    let minutes = seconds / 60;
    seconds %= 60;

    let minutes = minutes.try_into().unwrap();
    let seconds = seconds.try_into().unwrap();

    (minutes, seconds)
}
