use o2o::o2o;
use o2o::traits::TryIntoExisting;

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
#[try_map(NamedStruct, String)]
#[try_map(NamedStructModel, String)]
#[try_into_existing(NamedStruct, String)]
#[try_into_existing(NamedStructModel, String)]
struct NamedStructDto {
    some_int: i32,
    #[try_map(another_int)]
    diff_another_int: i32,
    #[map(NamedStruct| some_float)]
    #[map(NamedStructModel| some_float_diff)]
    diff_some_float: f32,
}

#[derive(Default, o2o)]
#[o2o(try_from(NamedStruct, i32))]
#[o2o(try_from(NamedStructModel, i32))]
struct UnnamedStructDto(
    #[o2o(try_map(some_int))] i32,
    #[o2o(map(another_int))] i32,
    #[o2o(map(NamedStruct| some_float), map(NamedStructModel| some_float_diff))] f32,
);

#[derive(Default, o2o)]
#[try_map(UnnamedStructDto, anyhow::Error)]
#[try_into_existing(UnnamedStructDto, anyhow::Error)]
struct NamedStruct2 {
    #[try_map(0)]
    some_int: i32,
    #[map(1)]
    another_int: i32,
    #[map(2)]
    some_float: f32,
}

#[test]
fn named2named() {
    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);

    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
    assert_eq!(456.0, model.some_float_diff);
}

#[test]
fn named2named_reverse() {
    let named = NamedStruct {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.diff_another_int);
    assert_eq!(456.0, dto.diff_some_float);

    let model = NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0,
    };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.diff_another_int);
    assert_eq!(456.0, dto.diff_some_float);
}

#[test]
fn named2named_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.diff_another_int);
    assert_eq!(named.some_float, dto.diff_some_float);

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
    assert_eq!(named.some_float, model.some_float_diff);
}

#[test]
fn named2named_ref_reversed() {
    let named = &NamedStruct {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.diff_another_int);
    assert_eq!(named.some_float, dto.diff_some_float);

    let model = &NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0,
    };

    let dto: NamedStructDto = model.try_into().unwrap();

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

    let unnamed: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);
    assert_eq!(456.0, unnamed.2);

    let model = NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0,
    };

    let unnamed: UnnamedStructDto = model.try_into().unwrap();

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

    let unnamed: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, unnamed.0);
    assert_eq!(named.another_int, unnamed.1);
    assert_eq!(named.some_float, unnamed.2);

    let model = &NamedStructModel {
        some_int: 123,
        another_int: 321,
        some_float_diff: 456.0,
    };

    let unnamed: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(model.some_int, unnamed.0);
    assert_eq!(model.another_int, unnamed.1);
    assert_eq!(model.some_float_diff, unnamed.2);
}

#[test]
fn unnamed2named() {
    let dto = UnnamedStructDto(123, 321, 456.0);

    let named: NamedStruct2 = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);
}

#[test]
fn unnamed2named_reverse() {
    let named = NamedStruct2 {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let dto: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456.0, dto.2);
}

#[test]
fn unnamed2named_ref() {
    let dto = &UnnamedStructDto(123, 321, 456.0);

    let named: NamedStruct2 = dto.try_into().unwrap();

    assert_eq!(dto.0, named.some_int);
    assert_eq!(dto.1, named.another_int);
    assert_eq!(dto.2, named.some_float);
}

#[test]
fn unnamed2named_reverse_ref() {
    let named = &NamedStruct2 {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let dto: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.0);
    assert_eq!(named.another_int, dto.1);
    assert_eq!(named.some_float, dto.2);
}

#[test]
fn existing_named2named() {
    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);

    let dto = NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
    assert_eq!(456.0, model.some_float_diff);
}

#[test]
fn existing_named2named_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        diff_another_int: 321,
        diff_some_float: 456.0,
    };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.diff_another_int);
    assert_eq!(named.some_float, dto.diff_some_float);

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
    assert_eq!(named.some_float, model.some_float_diff);
}

#[test]
fn existing_named2unnamed() {
    let named = NamedStruct2 {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let mut dto: UnnamedStructDto = Default::default();
    named.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456.0, dto.2);
}

#[test]
fn existing_named2unnamed_ref() {
    let named = &NamedStruct2 {
        some_int: 123,
        another_int: 321,
        some_float: 456.0,
    };

    let mut dto: UnnamedStructDto = Default::default();
    named.try_into_existing(&mut dto).unwrap();

    assert_eq!(named.some_int, dto.0);
    assert_eq!(named.another_int, dto.1);
    assert_eq!(named.some_float, dto.2);
}
