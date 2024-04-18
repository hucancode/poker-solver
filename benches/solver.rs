use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poker_solver::poker::Game;

pub fn bench_game_solver(c: &mut Criterion) {
    let mut game = Game::new();
    let mut hidden_4 = c.benchmark_group("4 cards hidden games");
    hidden_4.warm_up_time(Duration::from_secs(20))
        .measurement_time(Duration::from_secs(100));
    hidden_4.bench_function("community 8TQ hand 62", |b| {
        b.iter(|| game.solve_by(black_box("6s2h"), black_box(""), black_box("8cTdQh")))
    });
    hidden_4.bench_function("community 268 hand AA", |b| {
        b.iter(|| game.solve_by(black_box("AsAd"), black_box(""), black_box("2c6s8s")))
    });
    hidden_4.bench_function("community 268 hand TQ", |b| {
        b.iter(|| game.solve_by(black_box("TdQh"), black_box(""), black_box("2c6s8s")))
    });
    hidden_4.finish();
    let mut hidden_3 = c.benchmark_group("3 cards hidden games");
    hidden_3.warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(50));
    hidden_3.bench_function("community 8TQA hand 62", |b| {
        b.iter(|| game.solve_by(black_box("6s2h"), black_box(""), black_box("8cTdQhAs")))
    });
    hidden_3.bench_function("community 268K hand AA", |b| {
        b.iter(|| game.solve_by(black_box("AsAd"), black_box(""), black_box("2c6s8sKs")))
    });
    hidden_3.bench_function("community 268T hand TQ", |b| {
        b.iter(|| game.solve_by(black_box("TdQh"), black_box(""), black_box("2c6s8sTs")))
    });
    hidden_3.finish();
    let mut hidden_2 = c.benchmark_group("2 cards hidden games");
    hidden_2.warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(10));
    hidden_2.bench_function("community 8TQA6 hand 62", |b| {
        b.iter(|| game.solve_by(black_box("6s2h"), black_box(""), black_box("8cTdQhAs6d")))
    });
    hidden_2.bench_function("community 268K7 hand AA", |b| {
        b.iter(|| game.solve_by(black_box("AsAd"), black_box(""), black_box("2c6s8sKs7d")))
    });
    hidden_2.bench_function("community 268T3 hand TQ", |b| {
        b.iter(|| game.solve_by(black_box("TdQh"), black_box(""), black_box("2c6s8sTs3s")))
    });
    hidden_2.finish();
    let mut revealed = c.benchmark_group("revealed games");
    revealed.bench_function("community 8TQA6 hand 62 vs 99", |b| {
        b.iter(|| {
            game.solve_by(
                black_box("6s2h"),
                black_box("9d9h"),
                black_box("8cTdQhAs6d"),
            )
        })
    });
    revealed.bench_function("community 268K7 hand AA vs 8Q", |b| {
        b.iter(|| {
            game.solve_by(
                black_box("AsAd"),
                black_box("8dQh"),
                black_box("2c6s8sKs7d"),
            )
        })
    });
    revealed.bench_function("community 268T3 hand TQ vs KK", |b| {
        b.iter(|| {
            game.solve_by(
                black_box("TdQh"),
                black_box("KsKd"),
                black_box("2c6s8sTs3s"),
            )
        })
    });
    revealed.finish();
}
criterion_group!(game, bench_game_solver);
criterion_main!(game);
