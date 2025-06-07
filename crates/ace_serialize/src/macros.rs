// Based on a macro from serde_json
// https://github.com/serde-rs/json
// Copyright 2019-2020 David Tolnay
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT OR Apache-2.0

macro_rules! check_recursion {
    ($self:ident, $error:ident, $($body:tt)*) => {
        check_recursion!($self.remaining_depth, $error, $($body)*)
    };
    ($self:ident.$counter:ident, $error:ident, $($body:tt)*) => {
        $self.$counter -= 1;
        if $self.$counter == 0 {
            return Err($error::RecursionLimitExceeded);
        }

        $($body)*

        $self.$counter += 1;
    };
}

pub(crate) use check_recursion;
