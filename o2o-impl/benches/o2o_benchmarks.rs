use criterion::{black_box, criterion_group, criterion_main, Criterion};
use o2o_impl::expand::derive;
use quote::quote;
use syn::DeriveInput;

fn derive_simple_benchmark(c: &mut Criterion) {
    let tokens = quote! {
        #[map(NamedStructDto)]
        #[into_existing(NamedStructDto)]
        struct NamedStruct {
            some_int: i32,
            another_int: i32,
        }
    };

    let input: DeriveInput = syn::parse2(tokens).unwrap();
    c.bench_function("derive_simple_benchmark", |b| b.iter(|| derive(black_box(&input.clone()))));
}

fn derive_complex_benchmark(c: &mut Criterion) {
    let tokens = quote! {
        #[map(Entity as {})]
        #[map(TupleEntity)]
        #[into_existing(Entity as {})]
        #[into_existing(TupleEntity)]
        #[children(Entity| base: BaseEntity as {}, base.base: Base as {}, child: Child as {})]
        #[children(TupleEntity| 1: TupleBaseEntity, 1 .0: TupleBase, 2: TupleChild)]
        struct TupleEntityDto (
            #[map(Entity| parent_int)]
            i32,
            #[o2o(child(Entity| base.base))]
            #[o2o(child(TupleEntity| 1 .0))]
            #[o2o(map(Entity| base_int_2))]
            #[o2o(map(TupleEntity| 0))]
            i32,
            #[child(Entity| base.base)]
            #[child(TupleEntity| 1 .0)]
            #[map(Entity| another_base_int)]
            #[map(TupleEntity| 1)]
            i32,
            #[child(Entity| base)]
            #[child(TupleEntity| 1)]
            #[map(Entity| base_entity_int)]
            #[map(TupleEntity| 1)]
            i32,
            #[child(Entity| child)]
            #[child(TupleEntity| 2)]
            #[map(Entity| child_int)]
            #[map(TupleEntity| 0)]
            i32,
            #[child(Entity| child)]
            #[child(TupleEntity| 2)]
            #[map(Entity| another_child_int)]
            #[map(TupleEntity| 1)]
            i32,
        );
    };

    let input: DeriveInput = syn::parse2(tokens).unwrap();
    c.bench_function("derive_complex_benchmark", |b| b.iter(|| derive(black_box(&input.clone()))));
}

criterion_group!(benches, derive_simple_benchmark, derive_complex_benchmark);
criterion_main!(benches);
