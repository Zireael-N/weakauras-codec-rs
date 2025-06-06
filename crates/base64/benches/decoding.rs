use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_feature = "avx2", target_feature = "sse4.1")
))]
use weakauras_codec_base64::decode::arch::x86_64;
use weakauras_codec_base64::decode::{calculate_decoded_len, scalar};

extern crate alloc;
use alloc::vec::Vec;

pub fn decoding_benchmark(c: &mut Criterion) {
    #[allow(non_upper_case_globals)]
    const KiB: usize = 1024;

    let mut group = c.benchmark_group("decode");
    for size in [KiB, 2 * KiB, 4 * KiB, 8 * KiB, 16 * KiB, 1024 * KiB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        let data: Vec<_> = (b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .chain(b'0'..=b'9')
            .chain(b'('..=b')')
            .cycle()
            .take(*size)
            .collect();

        let capacity = calculate_decoded_len(&data).unwrap();

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter_batched_ref(
                || Vec::with_capacity(capacity),
                |buffer| unsafe {
                    scalar::decode_into_unchecked(&data, buffer.spare_capacity_mut())
                },
                BatchSize::SmallInput,
            );
        });

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("SSE4.1", size), size, |b, _| {
                b.iter_batched_ref(
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe {
                        x86_64::sse41::decode_into_unchecked(&data, buffer.spare_capacity_mut())
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
                        x86_64::avx2::decode_into_unchecked(&data, buffer.spare_capacity_mut())
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, decoding_benchmark);
criterion_main!(benches);
