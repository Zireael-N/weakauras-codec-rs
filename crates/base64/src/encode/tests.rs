// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

#[allow(unused_macros)]
macro_rules! base64_encode {
    ($input:expr, $output:ident, $module:ident) => {
        let buffer = $output.as_mut_vec();
        let len = $module::encode_into_unchecked($input, buffer.spare_capacity_mut());
        buffer.set_len(len);
    };
}

#[allow(unused_imports)]
pub(crate) use base64_encode;
