// Copyright 2024-2025 Irreducible Inc.

use std::{hint::black_box, iter::repeat_with};

use binius_fast_compute::arith_circuit::ArithCircuitPoly;
use binius_field::{
	BinaryField1b, Field, PackedBinaryField1x128b, PackedBinaryField16x8b, PackedBinaryField128x1b,
	PackedField,
};
use binius_math::{ArithExpr as Expr, CompositionPoly, RowsBatchRef};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rand::RngCore;

const BATCH_SIZE: usize = 256;

fn generate_random_vec<P: PackedField>(mut rng: impl RngCore) -> Vec<P> {
	repeat_with(|| P::random(&mut rng))
		.take(BATCH_SIZE)
		.collect()
}

fn generate_input_data<P: PackedField>(mut rng: impl RngCore) -> Vec<Vec<P>> {
	repeat_with(|| generate_random_vec(&mut rng))
		.take(4)
		.collect()
}

fn evaluate_arith_circuit_poly<P: PackedField>(
	query: &[&[P]],
	arith_circuit_poly: &impl CompositionPoly<P>,
) {
	for i in 0..BATCH_SIZE {
		let result = arith_circuit_poly
			.evaluate(black_box(&[
				black_box(query[0][i]),
				black_box(query[1][i]),
				black_box(query[2][i]),
				black_box(query[3][i]),
			]))
			.unwrap();
		let _ = black_box(result);
	}
}

fn benchmark_evaluate(c: &mut Criterion) {
	let mut rng = rand::rng();

	let query128x1b = generate_input_data(&mut rng);
	let query128x1b = query128x1b.iter().map(|q| q.as_slice()).collect::<Vec<_>>();
	let batch_query128x1b = RowsBatchRef::new(&query128x1b, BATCH_SIZE);
	let mut results128x1b = vec![PackedBinaryField128x1b::zero(); BATCH_SIZE];

	let query16x8b = generate_input_data(&mut rng);
	let query16x8b = query16x8b.iter().map(|q| q.as_slice()).collect::<Vec<_>>();
	let batch_query16x8b = RowsBatchRef::new(&query16x8b, BATCH_SIZE);
	let mut results16x8b = vec![PackedBinaryField16x8b::zero(); BATCH_SIZE];

	let query1x128b = generate_input_data(&mut rng);
	let query1x128b = query1x128b.iter().map(|q| q.as_slice()).collect::<Vec<_>>();
	let batch_query1x128b = RowsBatchRef::new(&query1x128b, BATCH_SIZE);
	let mut results1x128b = vec![PackedBinaryField1x128b::zero(); BATCH_SIZE];

	let arith_circuit_poly = ArithCircuitPoly::new(
		(Expr::Var(0) * Expr::Var(1)
			+ (Expr::Const(BinaryField1b::ONE) - Expr::Var(0)) * Expr::Var(2)
			- Expr::Var(3))
		.into(),
	);

	let mut group = c.benchmark_group("evaluate");
	group.throughput(Throughput::Elements(BATCH_SIZE as _));
	group.bench_function("arith_circuit_poly_128x1b", |bench| {
		bench.iter(|| {
			evaluate_arith_circuit_poly(&query128x1b, &arith_circuit_poly);
		});
	});
	group.bench_function("arith_circuit_poly_16x8b", |bench| {
		bench.iter(|| {
			evaluate_arith_circuit_poly(&query16x8b, &arith_circuit_poly);
		});
	});
	group.bench_function("arith_circuit_poly_1x128b", |bench| {
		bench.iter(|| {
			evaluate_arith_circuit_poly(&query1x128b, &arith_circuit_poly);
		});
	});
	group.finish();

	let mut group = c.benchmark_group("batch_evaluate");
	group.throughput(Throughput::Elements(BATCH_SIZE as _));
	group.bench_function("arith_circuit_poly_128x1b", |bench| {
		bench.iter(|| {
			arith_circuit_poly
				.batch_evaluate(&batch_query128x1b, &mut results128x1b)
				.unwrap();
		});
	});
	group.bench_function("arith_circuit_poly_16x8b", |bench| {
		bench.iter(|| {
			arith_circuit_poly
				.batch_evaluate(&batch_query16x8b, &mut results16x8b)
				.unwrap();
		});
	});
	group.bench_function("arith_circuit_poly_1x128b", |bench| {
		bench.iter(|| {
			arith_circuit_poly
				.batch_evaluate(&batch_query1x128b, &mut results1x128b)
				.unwrap();
		});
	});
	group.finish();
}

criterion_main!(arith_circuit_poly);
criterion_group!(arith_circuit_poly, benchmark_evaluate);
