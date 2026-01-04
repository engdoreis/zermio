// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Bitfield;
pub use crate::mmio::Device;
pub use crate::rdljson;
use svd_rs::cluster;
use svd_rs::registercluster;

#[derive(Debug)]
pub struct RegisterInfo {
    pub name: String,
    pub type_: String,
    pub desc: String,
    pub offset: u32,
}

impl RegisterInfo {
    pub fn new(
        name: impl Into<String>,
        type_name: Option<String>,
        desc: Option<String>,
        offset: u32,
    ) -> Self {
        let name = name.into();
        RegisterInfo {
            name: name.clone(),
            type_: type_name.unwrap_or(name.clone().replace("%s", "")),
            desc: desc.unwrap_or(name),
            offset,
        }
    }
}

#[derive(Debug)]
pub struct Register {
    pub info: Vec<RegisterInfo>, // Must have at least one.
    pub bitfields: Vec<Bitfield>,
}

impl Register {
    pub fn new(
        name: impl Into<String>,
        type_name: Option<String>,
        offset: u32,
        desc: Option<String>,
        bitfields: Vec<Bitfield>,
    ) -> Self {
        Self {
            info: vec![RegisterInfo::new(name, type_name, desc, offset)],
            bitfields,
        }
    }

    pub fn is_readable(&self) -> bool {
        self.bitfields.iter().any(|f| f.permissions.is_readable())
    }

    pub fn is_writable(&self) -> bool {
        self.bitfields.iter().any(|f| f.permissions.is_writable())
    }

    pub fn try_from(cluster: &cluster::Cluster) -> Result<Vec<Self>, String> {
        let cluster::Cluster::Array(info, dim) = cluster else {
            unreachable!()
        };

        let res = info
            .children
            .iter()
            .map(|children| {
                let registercluster::RegisterCluster::Register(register) = children else {
                    panic!("Too much recursion")
                };
                let mut dim = dim.clone();
                dim.dim_name = Some(info.name.clone());
                // This is a hack to reuse the `TryFrom<&svd_rs::register::Register> for Register`
                // and avoid reimplementing the cluster parsing. We rely on the fact that
                // the type `MaybeArray` implements `Deref`, so we deref register twice
                // to get in inner type (`RegisterInfo`), then we create a
                // `Register::Array` to be able to call `into`.
                let register = svd_rs::register::Register::Array((**register).clone(), dim);
                (&register).into()
            })
            .collect::<Vec<_>>();
        Ok(res)
    }
}

impl From<&svd_rs::register::RegisterInfo> for Register {
    fn from(register: &svd_rs::register::RegisterInfo) -> Self {
        let bitfields = if let Some(ref bitfields) = register.fields {
            bitfields
                .iter()
                .map(|field| {
                    field
                        .try_into()
                        .unwrap_or_else(|e| panic!("{} register {}", e, register.name))
                })
                .collect::<Vec<Bitfield>>()
        } else {
            vec![Bitfield::default()]
        };

        Self::new(
            register.name.clone(),
            None,
            register.address_offset,
            register.description.clone(),
            bitfields,
        )
    }
}

impl From<&svd_rs::register::Register> for Register {
    fn from(register: &svd_rs::register::Register) -> Self {
        let (mut register, dim): (Self, _) = match register {
            svd_rs::register::Register::Single(info) => return info.into(),
            svd_rs::register::Register::Array(info, dim) => (info.into(), dim),
        };

        let base_name = dim
            .dim_name
            .clone()
            .unwrap_or(register.info[0].name.clone());
        let type_name = register.info[0].type_.clone();

        let index: Vec<_> = dim
            .dim_index
            .clone()
            .unwrap_or((0..dim.dim).map(|n| n.to_string()).collect());
        let mut indexes = index.iter();
        let index = indexes.next().unwrap();
        register.info[0].name = Device::get_cluster_name(&base_name, index, &type_name);

        let mut offset = dim.dim_increment + register.info[0].offset;
        for index in indexes {
            let name = Device::get_cluster_name(&base_name, index, &type_name);
            register.info.push(RegisterInfo::new(
                name,
                Some(type_name.clone()),
                None,
                offset,
            ));
            offset += dim.dim_increment;
        }
        register
    }
}

impl From<&rdljson::Register> for Register {
    fn from(register: &rdljson::Register) -> Self {
        let fields: Vec<_> = register.fields.iter().map(|field| field.into()).collect();

        let name = format!(
            "{}{}",
            register.name.clone(),
            if register.offsets.len() > 1 { "0" } else { "" }
        );
        let mut this = Self::new(
            name,
            Some(register.type_name.clone()),
            register.offsets[0],
            register.desc.clone(),
            fields,
        );
        this.info.extend(
            register
                .offsets
                .iter()
                .enumerate()
                .skip(1)
                .map(|(idx, offset)| {
                    RegisterInfo::new(
                        format!("{}{}", register.name, idx),
                        Some(register.type_name.clone()),
                        register.desc.clone(),
                        *offset,
                    )
                }),
        );
        this
    }
}
