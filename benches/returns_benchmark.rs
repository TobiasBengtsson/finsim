use criterion::{black_box, criterion_group, criterion_main, Criterion};
use finsim::returns::{self, GenReturnsArgs, AccumulateArgs};

pub fn criterion_benchmark(c: &mut Criterion) {
    let gen_returns_args = GenReturnsArgs {
        total_seconds: Some(1000000),
        interval_seconds: None,
        num_points: 100000,
        yearly_mean: 1.0,
        yearly_stddev: 1.5,
        seed: None,
    };
    c.bench_function(
        "gen_returns 100000 data points",
        |b| b.iter(|| returns::gen_returns(black_box(&gen_returns_args)).collect::<Vec<f64>>()),
    );

    let accumulate_args = AccumulateArgs {
        accumulate: true,
        start_value: 100.0,
    };
    let ret_series = returns::gen_returns(black_box(&gen_returns_args)).collect::<Vec<f64>>();
    c.bench_function(
        "accumulate 100000 data points",
        |b| b.iter(|| returns::accumulate(black_box(ret_series.clone().into_iter()), &accumulate_args)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
