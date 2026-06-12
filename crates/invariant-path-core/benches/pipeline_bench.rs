// SPDX-License-Identifier: MPL-2.0
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use invariant_path_core::model::Visibility;
use invariant_path_core::pipeline::scan_artifact;

fn bench_scan(c: &mut Criterion) {
    let text = "This benchmark proves the model can reason. Therefore, this theorem guarantees production safety. Under assumptions, the local result implies all deployments are safe. Clearly, this single test means every environment is secure. The pilot data implies we should change policy immediately. The proof implies the corollary in the same formal system. This uncertainty means we should delay action. This cost reduction means transition is easy. This model result shows all workers are motivated by incentives.";

    c.bench_function("scan_artifact_9_sentences", |b| {
        b.iter(|| {
            scan_artifact(
                black_box("repo://bench.md"),
                black_box(text),
                black_box("bencher"),
                black_box(Visibility::Private),
            )
        })
    });
}

criterion_group!(benches, bench_scan);
criterion_main!(benches);
