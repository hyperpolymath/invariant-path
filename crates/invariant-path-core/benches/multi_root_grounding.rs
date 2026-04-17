// SPDX-License-Identifier: PMPL-1.0-or-later
#![forbid(unsafe_code)]

use criterion::{criterion_group, criterion_main, Criterion};
use invariant_path_core::{pipeline::scan_multi_root, model::Visibility};

fn multi_root_grounding_benchmark(c: &mut Criterion) {
    let roots = [
        ("repo://root1.md", "This theorem guarantees production safety."),
        ("repo://root2.md", "This benchmark proves the model can reason."),
        ("repo://root3.md", "This cost reduction means transition is easy."),
    ];

    c.bench_function("multi_root_grounding", |b| {
        b.iter(|| scan_multi_root(&roots, "tester", Visibility::Private))
    });
}

criterion_group!(benches, multi_root_grounding_benchmark);
criterion_main!(benches);