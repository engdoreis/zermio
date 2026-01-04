// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0
pub use crate::rdljson;
use svd_rs::access as svd_rs;

#[derive(Debug, Clone, Copy, Default, PartialEq, strum::Display, strum::IntoStaticStr)]
pub enum Permissions {
    #[default]
    ReadWrite,
    ReadWriteOnce,
    Read,
    Write,
    WriteOnce,
}

impl Permissions {
    pub fn is_readable(self) -> bool {
        self == Self::ReadWrite || self == Self::Read || self == Self::ReadWriteOnce
    }

    pub fn is_writable(self) -> bool {
        self == Self::ReadWrite
            || self == Self::WriteOnce
            || self == Self::Write
            || self == Self::ReadWriteOnce
    }
}

impl From<&str> for Permissions {
    fn from(s: &str) -> Self {
        match s {
            "read-only" => Self::Read,
            "write-only" => Self::Write,
            "read-write" => Self::ReadWrite,
            _ => panic!("{} unsuported", s),
        }
    }
}

impl From<svd_rs::Access> for Permissions {
    fn from(s: svd_rs::Access) -> Self {
        match s {
            svd_rs::Access::ReadOnly => Permissions::Read,
            svd_rs::Access::ReadWrite => Permissions::ReadWrite,
            svd_rs::Access::ReadWriteOnce => Permissions::ReadWriteOnce,
            svd_rs::Access::WriteOnce => Permissions::WriteOnce,
            svd_rs::Access::WriteOnly => Permissions::Write,
        }
    }
}

impl From<&rdljson::RegisterField> for Permissions {
    fn from(field: &rdljson::RegisterField) -> Self {
        match (field.sw_writable, field.sw_readable) {
            (false, true) => Permissions::Read,
            (true, true) => Permissions::ReadWrite,
            (true, false) => Permissions::Write,
            (false, false) => panic!("not allowed"),
        }
    }
}
