// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86_64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86_64::encode_into_unchecked;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use crate::encode::scalar::encode_into_unchecked;
