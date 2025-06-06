// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub(crate) fn base64_iter() -> impl Iterator<Item = u8> {
    (b'a'..=b'z')
        .chain(b'A'..=b'Z')
        .chain(b'0'..=b'9')
        .chain(b'('..=b')')
        .cycle()
}
