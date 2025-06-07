// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::decode_into_unchecked;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86_64::decode_into_unchecked;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
pub use crate::decode::scalar::decode_into_unchecked;
