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

use glib::clone;
use glib::subclass;
use glib::subclass::prelude::*;

// `Rc`s are Reference Counters. They allow us to clone objects,
// while actually referencing at different places.
// A `RefCell` allows for interior mutablility.
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc};

// OnceCell allows for a "nullable" field in a simple way.
use once_cell::sync::OnceCell;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LapType {
    Pomodoro,
    Break,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimerActions {
    CountdownUpdate(u32, u32),
    Lap(LapType),
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

#[derive(Debug)]
pub struct TimerPriv {
    state: Rc<RefCell<TimerState>>,
    instant: Rc<RefCell<Option<Instant>>>,
    duration: Rc<RefCell<Duration>>,
    sender: OnceCell<glib::Sender<TimerActions>>,
    lap_type: Rc<RefCell<LapType>>,
}

impl ObjectSubclass for TimerPriv {
    const NAME: &'static str = "SolanumTimer";
    type Type = Timer;
    type ParentType = glib::Object;
    type Interfaces = ();
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib::object_subclass!();

    fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(TimerState::default())),
            instant: Rc::new(RefCell::new(None)),
            duration: Rc::new(RefCell::new(Duration::new(0, 0))),
            sender: OnceCell::new(),
            lap_type: Rc::new(RefCell::new(LapType::Pomodoro)),
        }
    }
}

impl ObjectImpl for TimerPriv {}

glib::wrapper! {
    pub struct Timer(ObjectSubclass<TimerPriv>);
}

impl Timer {
    pub fn new(duration: u64, sender: glib::Sender<TimerActions>) -> Self {
        let obj: Self = glib::Object::new::<Self>(&[]).expect("Failed to initialize Timer object");
        let priv_ = obj.get_private();

        obj.set_duration(duration);
        priv_
            .sender
            .set(sender)
            .expect("Could not initialize sender");

        obj
    }

    fn get_private(&self) -> &TimerPriv {
        &TimerPriv::from_instance(self)
    }

    pub fn set_duration(&self, duration: u64) {
        let priv_ = self.get_private();

        let mut i = priv_.instant.borrow_mut();
        *i = Some(Instant::now());
        let mut d = priv_.duration.borrow_mut();
        *d = Duration::new(duration, 0);
    }

    pub fn start(&self) {
        let priv_ = self.get_private();

        let mut state = priv_.state.borrow_mut();
        *state = TimerState::Running;
        let mut instant = priv_.instant.borrow_mut();
        *instant = Some(Instant::now());

        let s = &priv_.state;
        let i = &priv_.instant;
        let d = &priv_.duration;
        let tx = priv_.sender.clone();
        let lt = &priv_.lap_type;
        // Every 100 milliseconds, this closure gets called in order to update the timer
        glib::timeout_add_local(
            std::time::Duration::from_millis(100),
            clone!(@weak s, @weak i, @weak d, @weak lt => @default-return glib::Continue(false), move || {
                let state = s.borrow_mut();
                let instant = i.borrow_mut();
                let duration = d.borrow_mut();
                let sender = tx.get().unwrap();
                let mut lap_type = lt.borrow_mut();

                if *state == TimerState::Running {
                    if let Some(instant) = *instant {
                        let elapsed = instant.elapsed();
                        if let Some(difference) = duration.checked_sub(elapsed) {
                            let msm = duration_to_ms(difference);
                            let _ = sender.send(TimerActions::CountdownUpdate(msm.0, msm.1));
                            return glib::Continue(true);
                        } else {
                            let new_lt = {
                                if *lap_type == LapType::Pomodoro {
                                    LapType::Break
                                } else {
                                    LapType::Pomodoro
                                }
                            };
                            *lap_type = new_lt;
                            let _ = sender.send(TimerActions::Lap(new_lt));
                            return glib::Continue(false);
                        }
                    }
                }
                glib::Continue(false)
            }),
        );
    }

    pub fn stop(&self) {
        let priv_ = self.get_private();

        let mut state = priv_.state.borrow_mut();
        *state = TimerState::Stopped;

        // When paused, set the timer so that it will resume where the user left off
        let mut duration = priv_.duration.borrow_mut();
        let instant = priv_.instant.borrow_mut().unwrap();
        let elapsed = instant.elapsed();
        if let Some(difference) = duration.checked_sub(elapsed) {
            *duration = difference;
        }

        println!("Timer stopped!")
    }

    pub fn set_lap_type(&self, new_type: LapType) {
        let priv_ = self.get_private();
        let mut lap_type = priv_.lap_type.borrow_mut();

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
