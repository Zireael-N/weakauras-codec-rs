use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use weakauras_codec_base64::*;

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

        let capacity = data.len() * 3 / 4;

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter_batched_ref(
                || Vec::with_capacity(capacity),
                |buffer| unsafe { decode_into_unchecked_scalar(&data, buffer) },
                BatchSize::SmallInput,
            );
        });

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("SSE", size), size, |b, _| {
                b.iter_batched_ref(
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe { decode_into_unchecked_sse(&data, buffer) },
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
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe { decode_into_unchecked_avx2(&data, buffer) },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, decoding_benchmark);
criterion_main!(benches);
