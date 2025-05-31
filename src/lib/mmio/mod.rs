// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub mod bitfield;
pub mod device;
pub mod interrupt;
pub mod permissions;
pub mod platform;
pub mod register;

pub use bitfield::Bitfield;
pub use device::Device;
pub use interrupt::Interrupt;
pub use permissions::Permissions;
pub use platform::Platform;
pub use register::Register;

static WIDTH: u32 = 32;
