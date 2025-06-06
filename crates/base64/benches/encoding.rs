use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_feature = "avx2", target_feature = "ssse3")
))]
use weakauras_codec_base64::encode::arch::x86_64;
use weakauras_codec_base64::encode::{calculate_encoded_len, scalar};

extern crate alloc;
use alloc::vec::Vec;

pub fn encoding_benchmark(c: &mut Criterion) {
    #[allow(non_upper_case_globals)]
    const KiB: usize = 1024;

    let mut group = c.benchmark_group("encode");
    for size in [KiB, 2 * KiB, 4 * KiB, 8 * KiB, 16 * KiB, 1024 * KiB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        let data: Vec<_> = (0u8..=255u8).cycle().take(*size).collect();

        let capacity = calculate_encoded_len(&data).unwrap();

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter_batched_ref(
                || Vec::with_capacity(capacity),
                |buffer| unsafe {
                    scalar::encode_into_unchecked(&data, buffer.spare_capacity_mut())
                },
                BatchSize::SmallInput,
            );
        });

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "ssse3"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("SSSE3", size), size, |b, _| {
                b.iter_batched_ref(
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe {
                        x86_64::ssse3::encode_into_unchecked(&data, buffer.spare_capacity_mut())
                    },
                    BatchSize::SmallInput,
                );
            });
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("AVX2", size), size, |b, _| {
                b.iter_batched_ref(
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe {
                        x86_64::avx2::encode_into_unchecked(&data, buffer.spare_capacity_mut())
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, encoding_benchmark);
criterion_main!(benches);
