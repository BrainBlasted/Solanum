// composite_template_derive.rs
//
// Copyright 2020 Christopher Davis <brainblasted@disroot.org>
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

use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::Data;

use std::string::ToString;

use crate::util::*;

fn gen_template_child_bindings(fields: &syn::Fields) -> TokenStream {
    let crate_ident = crate_ident_new();

    let recurse = fields.iter().map(|f| {
        let filtered_attrs = f
            .attrs
            .clone()
            .into_iter()
            .filter(|a| a.path.is_ident("template_child"))
            .collect::<Vec<syn::Attribute>>();
        if !filtered_attrs.is_empty() {
            let ident = f.ident.as_ref().unwrap();
            let mut value_id = String::new();

            if let Ok(attrs) = parse_template_child_attributes("template_child", &filtered_attrs) {
                attrs.into_iter().for_each(|a| match a {
                    TemplateChildAttribute::Id(id) => value_id = id.to_string(),
                });
            }

            quote! {
                Self::bind_template_child_with_offset(
                    klass,
                    &#value_id,
                    #crate_ident::offset_of!(Self => #ident),
                );
            }
        } else {
            quote! {}
        }
    });

    quote! {
        #(#recurse)*
    }
}

pub fn impl_composite_template(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    let fields = match input.data {
        Data::Struct(ref s) => &s.fields,
        _ => abort_call_site!("derive(CompositeTemplate) only supports structs"),
    };

    let template_children = gen_template_child_bindings(&fields);

    quote! {
        impl CompositeTemplate for #name {
            fn bind_template_children(klass: &mut Self::Class) {
                unsafe {
                    #template_children
                }
            }
        }
    }
}
