// Heavily inspired by a macro from memchr by BurntSushi
// https://github.com/BurntSushi/memchr
// Copyright 2018-2023 Andrew Gallant
// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

/// Macro for generating the body of a function that chooses
/// an implementation during runtime, based on available CPU features.
///
/// Feature detection is run only on the first call.
/// This is achieved by using an [AtomicPtr](core::sync::atomic::AtomicPtr)
/// to store the address of a specific implementation.
///
/// # Safety
///
/// * It must be used only on architectures where it is safe to transmute between
///   data pointers and function pointers, such as `x86_64`.
/// * The implementation of `$feature_module::$fn` must be safe to call on machines
///   where `$feature_name` is present.
/// * The implementation of `$fallback_module::$fn` must be safe to call on machines
///   where none of the provided features is present.
#[allow(unused_macros)]
macro_rules! unsafe_runtime_dispatch {
    (
        $fn:ident,
        $returnty:ty,
        $input:ident,
        $output:ident,
        $is_feature_detected:ident,
        $(($feature_name:tt, $feature_module:ident)),+,
        $fallback_module:ident,
    ) => {{
        #[cfg(feature = "std")]
        {
            use core::sync::atomic::{AtomicPtr, Ordering};

            type FnType = unsafe fn(&[u8], &mut [MaybeUninit<u8>]) -> $returnty;
            static FN_PTR: AtomicPtr<()> = AtomicPtr::new(init as *mut ());

            unsafe fn init(i: &[u8], o: &mut [MaybeUninit<u8>]) -> $returnty {
                let f = $(if std::$is_feature_detected!($feature_name) {
                    $feature_module::$fn as FnType
                } else)+ {
                    $fallback_module::$fn as FnType
                };

                FN_PTR.store(f as *mut (), Ordering::Relaxed);
                // SAFETY: We've chosen an implementation based on available CPU features.
                unsafe { f(i, o) }
            }

            let f = FN_PTR.load(Ordering::Relaxed);
            // SAFETY: According to the safety contract, this is used on an architecture
            // where transmuting between data pointers and function pointers is safe.
            unsafe { core::mem::transmute::<*mut (), FnType>(f)($input, $output) }
        }

        #[cfg(not(feature = "std"))]
        unsafe {
            $fallback_module::$fn($input, $output)
        }
    }}
}

#[allow(unused_imports)]
pub(crate) use unsafe_runtime_dispatch;
