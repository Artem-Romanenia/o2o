use criterion::{Criterion, criterion_group, criterion_main, black_box};
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
        #[map(Entity| base: BaseEntity, base.base: Base, child: Child)]
        #[map(TupleEntity as ()| 1: TupleBaseEntity as (), 1 .0: TupleBase as (), 2: TupleChild as ())]
        #[into_existing(Entity)]
        #[into_existing(TupleEntity as ())]
        struct EntityDto {
            #[map(TupleEntity| 0)]
            parent_int: i32,
            #[child(Entity| base.base)]
            #[child(TupleEntity| 1 .0)]
            #[map(Entity| base_int_2)]
            #[map(TupleEntity| 0)] 
            base_int: i32,
            #[child(Entity| base.base)]
            #[child(TupleEntity| 1 .0)]
            #[map(TupleEntity| 1)] 
            another_base_int: i32,
            #[child(Entity| base)]
            #[child(TupleEntity| 1)]
            #[map(TupleEntity| 1)] 
            base_entity_int: i32,
            #[child(Entity| child)]
            #[child(TupleEntity| 2)]
            #[map(TupleEntity| 0)] 
            child_int: i32,
            #[child(Entity| child)]
            #[child(TupleEntity| 2)]
            #[map(TupleEntity| 1)] 
            another_child_int: i32,
        }
    };

    let input: DeriveInput = syn::parse2(tokens).unwrap();
    c.bench_function("derive_complex_benchmark", |b| b.iter(|| derive(black_box(&input.clone()))));
}

criterion_group!(benches, derive_simple_benchmark, derive_complex_benchmark);
criterion_main!(benches);
