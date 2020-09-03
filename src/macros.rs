// macros.rs
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


macro_rules! add_style_class {
    ($widget:expr, @$name:ident) => {{
        let ctx = $widget.get_style_context();
        ctx.add_class(stringify!($name));
    }};
    ($widget:expr, $names:expr) => {{
        let ctx = $widget.get_style_context();
        for name in $names {
            ctx.add_class(name);
        }
    }};
}

macro_rules! remove_style_class {
    ($widget:expr, @$name:ident) => {{
        let ctx = $widget.get_style_context();
        ctx.remove_class(stringify!($name));
    }};
    ($widget:expr, $names:expr) => {{
        let ctx = $widget.get_style_context();
        for name in $names {
            ctx.remove_class(name);
        }
    }};
}
