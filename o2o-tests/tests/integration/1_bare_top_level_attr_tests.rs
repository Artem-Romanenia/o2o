use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct NamedStruct {
    some_int: i32,
    another_int: i32,
}

#[derive(Default)]
struct UnnamedStruct(i32, i32);

#[derive(Default)]
struct NamedStructModel {
    some_int: i32,
    another_int: i32,
}

struct UnnamedStructModel(i32, i32);

/// Test proper #[doc = ...] handling
#[derive(o2o)]
#[map(NamedStruct)]
#[map(NamedStructModel)]
#[into_existing(NamedStruct)]
#[into_existing(NamedStructModel)]
struct NamedStructDto {
    /// Test proper #[doc = ...] handling
    some_int: i32,
    another_int: i32,
}

#[derive(o2o)]
#[map(UnnamedStruct)]
#[map(UnnamedStructModel)]
#[into_existing(UnnamedStruct)]
#[into_existing(UnnamedStructModel)]
struct UnnamedStructDto(i32, i32);

#[test]
fn named2named() {
    let dto = NamedStructDto { some_int: 123, another_int: 321 };

    let named: NamedStruct = dto.into();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);

    let dto = NamedStructDto { some_int: 123, another_int: 321 };

    let model: NamedStructModel = dto.into();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
}

#[test]
fn named2named_reverse() {
    let named = NamedStruct { some_int: 123, another_int: 321 };

    let dto: NamedStructDto = named.into();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.another_int);

    let model = NamedStructModel { some_int: 123, another_int: 321 };

    let dto: NamedStructDto = model.into();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.another_int);
}

#[test]
fn named2named_ref() {
    let dto = &NamedStructDto { some_int: 123, another_int: 321 };

    let named: NamedStruct = dto.into();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.another_int);

    let model: NamedStructModel = dto.into();

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
}

#[test]
fn named2named_ref_reversed() {
    let named = &NamedStruct { some_int: 123, another_int: 321 };

    let dto: NamedStructDto = named.into();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.another_int);

    let model = &NamedStructModel { some_int: 123, another_int: 321 };

    let dto: NamedStructDto = model.into();

    assert_eq!(model.some_int, dto.some_int);
    assert_eq!(model.another_int, dto.another_int);
}

#[test]
fn unnamed2unnamed() {
    let dto = UnnamedStructDto(123, 321);

    let unnamed: UnnamedStruct = dto.into();

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);

    let dto = UnnamedStructDto(123, 321);

    let model: UnnamedStructModel = dto.into();

    assert_eq!(123, model.0);
    assert_eq!(321, model.1);
}

#[test]
fn unnamed2unnamed_reversed() {
    let unnamed = UnnamedStruct(123, 321);

    let dto: UnnamedStructDto = unnamed.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);

    let model = UnnamedStructModel(123, 321);

    let dto: UnnamedStructDto = model.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
}

#[test]
fn unnamed2unnamed_ref() {
    let dto = &UnnamedStructDto(123, 321);

    let unnamed: UnnamedStruct = dto.into();

    assert_eq!(dto.0, unnamed.0);
    assert_eq!(dto.1, unnamed.1);

    let model: UnnamedStructModel = dto.into();

    assert_eq!(dto.0, model.0);
    assert_eq!(dto.1, model.1);
}

#[test]
fn unnamed2unnamed_ref_reversed() {
    let unnamed = &UnnamedStruct(123, 321);

    let dto: UnnamedStructDto = unnamed.into();

    assert_eq!(unnamed.0, dto.0);
    assert_eq!(unnamed.1, dto.1);

    let model = &UnnamedStructModel(123, 321);

    let dto: UnnamedStructDto = model.into();

    assert_eq!(model.0, dto.0);
    assert_eq!(model.1, dto.1);
}

#[test]
fn existing_named2named() {
    let dto = NamedStructDto { some_int: 123, another_int: 321 };

    let mut named: NamedStruct = Default::default();
    dto.into_existing(&mut named);

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);

    let dto = NamedStructDto { some_int: 123, another_int: 321 };

    let mut model: NamedStructModel = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
}

#[test]
fn existing_named2named_ref() {
    let dto = &NamedStructDto { some_int: 123, another_int: 321 };

    let mut named: NamedStruct = Default::default();
    dto.into_existing(&mut named);

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.another_int, named.another_int);

    let dto = &NamedStructDto { some_int: 123, another_int: 321 };

    let mut model: NamedStructModel = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(dto.some_int, model.some_int);
    assert_eq!(dto.another_int, model.another_int);
}

#[test]
fn existing_unnamed2unnamed() {
    let dto = UnnamedStructDto(123, 321);

    let mut unnamed: UnnamedStruct = Default::default();

    dto.into_existing(&mut unnamed);

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);

    let dto = UnnamedStructDto(123, 321);

    let model: UnnamedStructModel = dto.into();

    assert_eq!(123, model.0);
    assert_eq!(321, model.1);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let dto = &UnnamedStructDto(123, 321);

    let mut unnamed: UnnamedStruct = Default::default();

    dto.into_existing(&mut unnamed);

    assert_eq!(dto.0, unnamed.0);
    assert_eq!(dto.1, unnamed.1);

    let dto = &UnnamedStructDto(123, 321);

    let model: UnnamedStructModel = dto.into();

    assert_eq!(dto.0, model.0);
    assert_eq!(dto.1, model.1);
}
