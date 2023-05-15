use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct NamedStruct {
    some_int: i32,
    another_int: i32,
    some_float: f32,
}

#[derive(Default)]
struct NamedStructModel {
    some_int: i32,
    another_int: i32,
    some_float_diff: f32,
}

#[derive(o2o)]
#[map(NamedStruct)]
#[map(NamedStructModel)]
#[into_existing(NamedStruct)]
#[into_existing(NamedStructModel)]
struct NamedStructDto {
    some_int: i32,
    #[map(another_int)] 
    diff_another_int: i32,
    #[map(NamedStruct| some_float)] 
    #[map(NamedStructModel| some_float_diff)]
    diff_some_float: f32,
}

#[derive(o2o)]
#[o2o(from(NamedStruct))]
#[o2o(from(NamedStructModel))]
struct UnnamedStructDto(
    #[o2o(map(some_int))] i32, 
    #[o2o(map(another_int))] i32, 
    #[o2o(map(NamedStruct| some_float); map(NamedStructModel| some_float_diff))] f32
);

#[test]
fn named2named() {
    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let named: NamedStruct = dto.into();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);

    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let model: NamedStructModel = dto.into();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
    assert_eq!(456.0, model.some_float_diff);
}

#[test]
fn named2named_reverse() {
    let named = NamedStruct {
        some_int: 123,
        another_int: 321,
        some_float: 456.0
    };

    let dto: NamedStructDto = named.into();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.diff_another_int);
    assert_eq!(456.0, dto.diff_some_float);

    let model = NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0
    };

    let dto: NamedStructDto = model.into();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.diff_another_int);
    assert_eq!(456.0, dto.diff_some_float);
}

#[test]
fn named2named_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0
    };

    let named: NamedStruct = dto.into();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.diff_another_int);
    assert_eq!(named.some_float, dto.diff_some_float);

    let model: NamedStructModel = dto.into();

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
    assert_eq!(named.some_float, model.some_float_diff);
}

#[test]
fn named2named_ref_reversed() {
    let named = &NamedStruct {
        some_int: 123,
        another_int: 321,
        some_float: 456.0
    };

    let dto: NamedStructDto = named.into();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.diff_another_int);
    assert_eq!(named.some_float, dto.diff_some_float);

    let model = &NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0
    };

    let dto: NamedStructDto = model.into();

    assert_eq!(model.some_int, dto.some_int);
    assert_eq!(model.another_int, dto.diff_another_int);
    assert_eq!(model.some_float_diff, dto.diff_some_float);
}

#[test] 
fn named2unnamed() {
    let named = NamedStruct {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let unnamed: UnnamedStructDto = named.into();

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);
    assert_eq!(456.0, unnamed.2);

    let model = NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0,
    };

    let unnamed: UnnamedStructDto = model.into();

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);
    assert_eq!(456.0, unnamed.2);
}

#[test] 
fn named2unnamed_ref() {
    let named = &NamedStruct {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let unnamed: UnnamedStructDto = named.into();

    assert_eq!(named.some_int, unnamed.0);
    assert_eq!(named.another_int, unnamed.1);
    assert_eq!(named.some_float, unnamed.2);

    let model = &NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0,
    };

    let unnamed: UnnamedStructDto = named.into();

    assert_eq!(model.some_int, unnamed.0);
    assert_eq!(model.another_int, unnamed.1);
    assert_eq!(model.some_float_diff, unnamed.2);
}

#[test]
fn existing_named2named() {
    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let mut named: NamedStruct = Default::default();
    dto.into_existing(&mut named);

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);

    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let mut model: NamedStructModel = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
    assert_eq!(456.0, model.some_float_diff);
}

#[test]
fn existing_named2named_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0
    };

    let mut named: NamedStruct = Default::default();
    dto.into_existing(&mut named);

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.diff_another_int);
    assert_eq!(named.some_float, dto.diff_some_float);

    let mut model: NamedStructModel = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
    assert_eq!(named.some_float, model.some_float_diff);
}