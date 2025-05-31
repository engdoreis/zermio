// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Bitfield;
use svd_rs::cluster;
use svd_rs::register;
use svd_rs::registercluster;

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
            type_: type_name.unwrap_or(name.clone()),
            desc: desc.unwrap_or(name),
            offset: offset,
        }
    }
}

pub struct Register {
    pub info: Vec<RegisterInfo>, // Must have at least one.
    pub bitfields: Vec<Bitfield>,
}

impl Register {
    pub fn new(
        name: impl Into<String>,
        offset: u32,
        desc: Option<String>,
        bitfields: Vec<Bitfield>,
    ) -> Self {
        Self {
            info: vec![RegisterInfo::new(name, None, desc, offset)],
            bitfields,
        }
    }

    pub fn try_from(cluster: &cluster::Cluster) -> Result<Vec<Self>, String> {
        match cluster {
            cluster::Cluster::Array(info, dim) => {
                let mut res = Vec::new();
                let index = dim
                    .dim_index
                    .clone()
                    .unwrap_or((0..dim.dim).map(|n| n.to_string()).collect::<Vec<_>>());
                for children in info.children.iter() {
                    let mut indexes = index.iter();
                    match children {
                        registercluster::RegisterCluster::Register(register) => {
                            let mut register: Register = register.try_into()?;
                            let index = indexes.next().unwrap();
                            let type_name = dim
                                .dim_name
                                .clone()
                                .unwrap_or(register.info[0].name.clone());
                            register.info[0].name =
                                super::device_cluster_name(&info.name, &index, &type_name);
                            let mut offset = dim.dim_increment + register.info[0].offset;
                            for index in indexes {
                                let name =
                                    super::device_cluster_name(&info.name, &index, &type_name);
                                register.info.push(RegisterInfo::new(
                                    name,
                                    Some(type_name.clone()),
                                    None,
                                    offset,
                                ));
                                offset += dim.dim_increment;
                            }
                            res.push(register);
                        }
                        registercluster::RegisterCluster::Cluster(_) => {
                            panic!("Too much recursion")
                        }
                    }
                }
                return Ok(res);
            }
            cluster::Cluster::Single(_) => unreachable!(),
        }
    }
}

impl TryFrom<&register::Register> for Register {
    type Error = String;
    fn try_from(register: &register::Register) -> Result<Self, Self::Error> {
        let (register, dim) = match register {
            register::Register::Single(info) => (info, None),
            register::Register::Array(info, dim) => (info, Some(dim)),
        };

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

        let mut register = Self::new(
            register.name.clone(),
            register.address_offset as u32,
            register.description.clone(),
            bitfields,
        );

        if let Some(dim) = dim {
            // TODO: repeated code.
            let index = dim
                .dim_index
                .clone()
                .unwrap_or((0..dim.dim).map(|n| n.to_string()).collect::<Vec<_>>());
            let mut indexes = index.iter();
            let index = indexes.next().unwrap();
            let type_name = dim
                .dim_name
                .clone()
                .unwrap_or(register.info[0].name.clone());
            register.info[0].name = super::device_cluster_name(&type_name, &index, "");
            let mut offset = dim.dim_increment + register.info[0].offset;
            for index in indexes {
                let name = super::device_cluster_name(&type_name, &index, "");
                register.info.push(RegisterInfo::new(
                    name,
                    Some(type_name.clone()),
                    None,
                    offset,
                ));
                offset += dim.dim_increment;
            }
        }
        Ok(register)
    }
}
