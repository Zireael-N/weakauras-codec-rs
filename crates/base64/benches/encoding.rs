use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use weakauras_codec_base64::*;

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
                || String::with_capacity(capacity),
                |buffer| unsafe { encode_into_unchecked_scalar(&data, buffer) },
                BatchSize::SmallInput,
            );
        });

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "ssse3"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("SSE", size), size, |b, _| {
                b.iter_batched_ref(
                    || String::with_capacity(capacity),
                    |buffer| unsafe { encode_into_unchecked_sse(&data, buffer) },
                    BatchSize::SmallInput,
                );
            });
        }

        #[cfg(all(
            feature = "avx2",
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("AVX2", size), size, |b, _| {
                b.iter_batched_ref(
                    || String::with_capacity(capacity),
                    |buffer| unsafe { encode_into_unchecked_avx2(&data, buffer) },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

fn calculate_encoded_len(data: &[u8]) -> Option<usize> {
    let len = data.len();
    let leftover = len % 3;

    (len / 3).checked_mul(4).and_then(|len| {
        if leftover > 0 {
            len.checked_add(leftover + 1)
        } else {
            Some(len)
        }
    })
}

criterion_group!(benches, encoding_benchmark);
criterion_main!(benches);
