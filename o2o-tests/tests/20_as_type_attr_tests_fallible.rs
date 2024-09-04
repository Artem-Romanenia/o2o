use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(Default)]
struct NamedStruct {
    some_int: i32,
    another_int: i32,
    some_float: f32,
    another_float: f64,
}

#[derive(Default)]
struct NamedStructModel {
    some_int: i32,
    different_int: i8,
    some_float: f32,
    another_float: f64,
}

#[derive(o2o)]
#[try_map(NamedStruct, String)]
#[try_map(NamedStructModel, String)]
#[try_into_existing(NamedStruct, String)]
#[try_into_existing(NamedStructModel, String)]
struct NamedStructDto {
    some_int: i32,
    #[o2o(
        as_type(NamedStruct| another_int, i32),
        as_type(NamedStructModel| i8),
    )]
    different_int: i16,
    #[o2o(as_type(f32))]
    some_float: f64,
    #[o2o(as_type(another_float, f64))]
    different_float: f32,
}

#[test]
fn named2named_different_types() {
    let dto = NamedStructDto {
        some_int: 123,
        different_int: 321,
        some_float: 456.0,
        different_float: 654.0,
    };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);
    assert_eq!(654.0, named.another_float);

    let dto = NamedStructDto {
        some_int: 123,
        different_int: 127,
        some_float: 456.0,
        different_float: 654.0,
    };

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(127, model.different_int);
    assert_eq!(456.0, model.some_float);
    assert_eq!(654.0, model.another_float);
}

#[test]
fn named2named_different_types_reverse() {
    let named = NamedStruct { some_int: 123, another_int: 321, some_float: 456.0, another_float: 654.0 };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.different_int);
    assert_eq!(456.0, dto.some_float);

    let model = NamedStructModel { some_int: 123, different_int: 127, some_float: 456.0, another_float: 654.0 };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(127, dto.different_int);
    assert_eq!(456.0, dto.some_float);
    assert_eq!(654.0, dto.different_float);
}

#[test]
fn named2named_different_types_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        different_int: 127,
        some_float: 456.0,
        different_float: 654.0,
    };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.different_int, named.another_int as i16);
    assert_eq!(dto.some_float, named.some_float as f64);
    assert_eq!(dto.different_float, named.another_float as f32);

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(dto.some_int, model.some_int);
    assert_eq!(dto.different_int, model.different_int as i16);
    assert_eq!(dto.some_float, model.some_float as f64);
    assert_eq!(dto.different_float, model.another_float as f32);
}

#[test]
fn named2named_different_types_reverse_ref() {
    let named = &NamedStruct { some_int: 123, another_int: 321, some_float: 456.0, another_float: 654.0 };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.different_int as i32);
    assert_eq!(named.some_float, dto.some_float as f32);
    assert_eq!(named.another_float, dto.different_float as f64);

    let model = &NamedStructModel { some_int: 123, different_int: 127, some_float: 456.0, another_float: 654.0 };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(model.some_int, dto.some_int);
    assert_eq!(model.different_int, dto.different_int as i8);
    assert_eq!(model.some_float, dto.some_float as f32);
    assert_eq!(model.another_float, dto.different_float as f64);
}

#[test]
fn existing_named2named_different_types() {
    let dto = NamedStructDto {
        some_int: 123,
        different_int: 321,
        some_float: 456.0,
        different_float: 654.0,
    };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);
    assert_eq!(654.0, named.another_float);

    let dto = NamedStructDto {
        some_int: 123,
        different_int: 127,
        some_float: 456.0,
        different_float: 654.0,
    };

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(127, model.different_int);
    assert_eq!(456.0, model.some_float);
    assert_eq!(654.0, model.another_float);
}

#[test]
fn existing_named2named_different_types_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        different_int: 127,
        some_float: 456.0,
        different_float: 654.0,
    };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.different_int, named.another_int as i16);
    assert_eq!(dto.some_float, named.some_float as f64);
    assert_eq!(dto.different_float, named.another_float as f32);

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(dto.some_int, model.some_int);
    assert_eq!(dto.different_int, model.different_int as i16);
    assert_eq!(dto.some_float, model.some_float as f64);
    assert_eq!(dto.different_float, model.another_float as f32);
}
