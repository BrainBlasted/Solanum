// templating.rs
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
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::IsA;
use gtk::subclass::widget as wdgt;

pub fn get_template_child<W, T: IsA<glib::Object>>(widget: &W, name: &str) -> Option<T>
where
    W: IsA<gtk::Widget>,
{
    unsafe {
        Option::<glib::Object>::from_glib_none(gtk_sys::gtk_widget_get_template_child(
            widget.upcast_ref().as_ptr(),
            widget.get_type().to_glib(),
            name.to_glib_none().0,
        ))
        .and_then(|obj| obj.dynamic_cast::<T>().ok())
    }
}

pub trait WidgetSubclass
where
    Self: ObjectSubclass + wdgt::WidgetImpl,
{
    unsafe fn set_template_bytes(
        klass: &mut subclass::simple::ClassStruct<Self>,
        template: &glib::Bytes,
    ) {
        let type_class = klass as *mut _ as *mut gobject_sys::GTypeClass;
        let widget_class =
            gobject_sys::g_type_check_class_cast(type_class, gtk_sys::gtk_widget_get_type())
                as *mut gtk_sys::GtkWidgetClass;
        gtk_sys::gtk_widget_class_set_template(widget_class, template.to_glib_none().0);
    }

    unsafe fn set_template(klass: &mut subclass::simple::ClassStruct<Self>, template: &[u8]) {
        let template_bytes = glib::Bytes::from(template);
        Self::set_template_bytes(klass, &template_bytes);
    }

    unsafe fn set_template_static(
        klass: &mut subclass::simple::ClassStruct<Self>,
        template: &'static [u8],
    ) {
        let template_bytes = glib::Bytes::from_static(template);
        Self::set_template_bytes(klass, &template_bytes);
    }

    unsafe fn set_template_from_resource(
        klass: &mut subclass::simple::ClassStruct<Self>,
        resource_name: &str,
    ) {
        let type_class = klass as *mut _ as *mut gobject_sys::GTypeClass;
        let widget_class =
            gobject_sys::g_type_check_class_cast(type_class, gtk_sys::gtk_widget_get_type())
                as *mut gtk_sys::GtkWidgetClass;
        gtk_sys::gtk_widget_class_set_template_from_resource(
            widget_class,
            resource_name.to_glib_none().0,
        );
    }

    unsafe fn bind_template_child(klass: &mut subclass::simple::ClassStruct<Self>, name: &str) {
        let type_class = klass as *mut _ as *mut gobject_sys::GTypeClass;
        let widget_class =
            gobject_sys::g_type_check_class_cast(type_class, gtk_sys::gtk_widget_get_type())
                as *mut gtk_sys::GtkWidgetClass;
        gtk_sys::gtk_widget_class_bind_template_child_full(
            widget_class,
            name.to_glib_none().0,
            false as glib_sys::gboolean,
            0,
        );
    }

    unsafe fn bind_template_child_with_offset<T>(
        klass: &mut subclass::simple::ClassStruct<Self>,
        name: &str,
        offset: field_offset::FieldOffset<Self, TemplateWidget<T>>,
    ) where
        T: ObjectType + FromGlibPtrNone<*mut <T as ObjectType>::GlibType>,
    {
        let type_class = klass as *mut _ as *mut gobject_sys::GTypeClass;
        let widget_class =
            gobject_sys::g_type_check_class_cast(type_class, gtk_sys::gtk_widget_get_type())
                as *mut gtk_sys::GtkWidgetClass;
        let private_offset = Self::type_data().as_ref().private_offset;
        gtk_sys::gtk_widget_class_bind_template_child_full(
            widget_class,
            name.to_glib_none().0,
            false as glib_sys::gboolean,
            private_offset + (offset.get_byte_offset() as isize),
        )
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TemplateWidget<T>
where
    T: ObjectType + FromGlibPtrNone<*mut <T as ObjectType>::GlibType>,
{
    ptr: *mut <T as ObjectType>::GlibType,
}

impl<T> Default for TemplateWidget<T>
where
    T: ObjectType + FromGlibPtrNone<*mut <T as ObjectType>::GlibType>,
{
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }
}

#[allow(unused)]
impl<T> TemplateWidget<T>
where
    T: ObjectType + FromGlibPtrNone<*mut <T as ObjectType>::GlibType>,
{
    fn new() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    #[track_caller]
    pub fn get(&self) -> T {
        unsafe {
            Option::<T>::from_glib_none(self.ptr)
                .expect("Failed to retrieve template child. Please check that it has been bound.")
        }
    }
}
