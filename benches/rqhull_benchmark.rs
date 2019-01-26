#[macro_use]
extern crate criterion;
extern crate rqhull;

#[macro_use]
extern crate ndarray;

use criterion::Criterion;
use rqhull::Voronoi;
use ndarray::Array2;
use criterion::Bencher;
use rqhull::QhullError;


fn criterion_benchmark(c: &mut Criterion) {



    c.bench_function("voronoi 9", |b: &mut Bencher| {
        let mut base:Array2<f64> =   array![[0.0, 0.0],
                                            [0.0, 1.0],
                                            [0.0, 2.0],
                                            [1.0, 0.0],
                                            [1.0, 1.0],
                                            [1.0, 2.0],
                                            [2.0, 0.0],
                                            [2.0, 1.0],
                                            [2.0, 2.0]];
        let pts:&mut Array2<f64>  = &mut base;
        b.iter(|| {Voronoi::new(pts).expect("conversion failed");})
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
