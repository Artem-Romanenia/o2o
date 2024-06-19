use std::num::TryFromIntError;

use o2o::o2o;
use o2o::traits::TryIntoExisting;

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
#[try_map(NamedStruct, anyhow::Error)]
#[try_map(NamedStructModel, anyhow::Error)]
#[try_into_existing(NamedStruct, anyhow::Error)]
#[try_into_existing(NamedStructModel, anyhow::Error)]
struct NamedStructDto {
    /// Test proper #[doc = ...] handling
    some_int: i32,
    another_int: i32,
}

#[derive(o2o)]
#[try_map(UnnamedStruct, TryFromIntError)]
#[try_map(UnnamedStructModel, TryFromIntError)]
#[try_into_existing(UnnamedStruct, TryFromIntError)]
#[try_into_existing(UnnamedStructModel, TryFromIntError)]
struct UnnamedStructDto(i32, i32);


#[test]
fn named2named() {
    let dto = NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);

    let dto = NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
}

#[test]
fn named2named_reverse() {
    let named = NamedStruct {
        some_int: 123,
        another_int: 321,
    };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.another_int);

    let model = NamedStructModel {
        some_int: 123,
        another_int: 321,
    };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.another_int);
}

#[test]
fn named2named_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.another_int);

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
}

#[test]
fn named2named_ref_reversed() {
    let named = &NamedStruct {
        some_int: 123,
        another_int: 321,
    };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.another_int);

    let model = &NamedStructModel {
        some_int: 123,
        another_int: 321,
    };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(model.some_int, dto.some_int);
    assert_eq!(model.another_int, dto.another_int);
}

#[test]
fn unnamed2unnamed() {
    let dto = UnnamedStructDto(123, 321);

    let unnamed: UnnamedStruct = dto.try_into().unwrap();

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);

    let dto = UnnamedStructDto(123, 321);

    let model: UnnamedStructModel = dto.try_into().unwrap();

    assert_eq!(123, model.0);
    assert_eq!(321, model.1);
}

#[test]
fn unnamed2unnamed_reversed() {
    let unnamed = UnnamedStruct(123, 321);

    let dto: UnnamedStructDto = unnamed.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);

    let model = UnnamedStructModel(123, 321);

    let dto: UnnamedStructDto = model.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
}

#[test]
fn unnamed2unnamed_ref() {
    let dto = &UnnamedStructDto(123, 321);

    let unnamed: UnnamedStruct = dto.try_into().unwrap();

    assert_eq!(dto.0, unnamed.0);
    assert_eq!(dto.1, unnamed.1);

    let model: UnnamedStructModel = dto.try_into().unwrap();

    assert_eq!(dto.0, model.0);
    assert_eq!(dto.1, model.1);
}

#[test]
fn unnamed2unnamed_ref_reversed() {
    let unnamed = &UnnamedStruct(123, 321);

    let dto: UnnamedStructDto = unnamed.try_into().unwrap();

    assert_eq!(unnamed.0, dto.0);
    assert_eq!(unnamed.1, dto.1);

    let model = &UnnamedStructModel(123, 321);

    let dto: UnnamedStructDto = model.try_into().unwrap();

    assert_eq!(model.0, dto.0);
    assert_eq!(model.1, dto.1);
}

#[test]
fn existing_named2named() {
    let dto = NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);

    let dto = NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
}

#[test]
fn existing_named2named_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.another_int, named.another_int);

    let dto = &NamedStructDto {
        some_int: 123,
        another_int: 321,
    };

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(dto.some_int, model.some_int);
    assert_eq!(dto.another_int, model.another_int);
}

#[test]
fn existing_unnamed2unnamed() {
    let dto = UnnamedStructDto (123, 321);

    let mut unnamed: UnnamedStruct = Default::default();

    dto.try_into_existing(&mut unnamed).unwrap();

    assert_eq!(123, unnamed.0);
    assert_eq!(321, unnamed.1);

    let dto = UnnamedStructDto (123, 321);

    let model: UnnamedStructModel = dto.try_into().unwrap();

    assert_eq!(123, model.0);
    assert_eq!(321, model.1);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let dto = &UnnamedStructDto (123, 321);

    let mut unnamed: UnnamedStruct = Default::default();

    dto.try_into_existing(&mut unnamed).unwrap();

    assert_eq!(dto.0, unnamed.0);
    assert_eq!(dto.1, unnamed.1);

    let dto = &UnnamedStructDto (123, 321);

    let model: UnnamedStructModel = dto.try_into().unwrap();

    assert_eq!(dto.0, model.0);
    assert_eq!(dto.1, model.1);
}