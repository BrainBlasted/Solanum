// util.rs
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

use anyhow::{bail, Result};
use itertools::Itertools;
use syn::{Attribute, Lit, Meta, MetaList, NestedMeta};

// find the #[@attr_name] attribute in @attrs
fn find_attribute_meta(attrs: &[Attribute], attr_name: &str) -> Result<Option<MetaList>> {
    let meta = match attrs.iter().find(|a| a.path.is_ident(attr_name)) {
        Some(a) => a.parse_meta(),
        _ => return Ok(None),
    };
    match meta? {
        Meta::List(n) => Ok(Some(n)),
        _ => bail!("wrong meta type"),
    }
}

#[derive(Debug)]
pub enum TemplateChildAttribute {
    Id(String),
}

fn parse_attribute(meta: &NestedMeta) -> Result<(String, String)> {
    let meta = match &meta {
        NestedMeta::Meta(m) => m,
        _ => bail!("wrong meta type: not a NestedMeta::Meta"),
    };
    let meta = match meta {
        Meta::NameValue(n) => n,
        _ => bail!("wrong meta type: not a Meta::NameValue"),
    };
    let value = match &meta.lit {
        Lit::Str(s) => s.value(),
        _ => bail!("wrong meta type: not a Lit::Str"),
    };

    let ident = match meta.path.get_ident() {
        None => bail!("missing ident"),
        Some(ident) => ident,
    };

    Ok((ident.to_string(), value))
}

fn parse_template_child_attribute(meta: &NestedMeta) -> Result<TemplateChildAttribute> {
    let (ident, v) = parse_attribute(meta)?;

    match ident.as_ref() {
        "id" => Ok(TemplateChildAttribute::Id(v)),
        s => bail!("Unknown item meta {}", s),
    }
}

pub fn parse_template_child_attributes(
    attr_name: &str,
    attrs: &[Attribute],
) -> Result<Vec<TemplateChildAttribute>> {
    let meta_list = find_attribute_meta(attrs, attr_name)?;
    let v = match meta_list {
        Some(meta) => meta
            .nested
            .iter()
            .map(|m| parse_template_child_attribute(&m))
            .fold_results(Vec::new(), |mut v, a| {
                v.push(a);
                v
            })?,
        None => Vec::new(),
    };

    Ok(v)
}
