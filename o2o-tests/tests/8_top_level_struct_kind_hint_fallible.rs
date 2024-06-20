use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(Default)]
struct UnnamedStructDto(i32, i32, f32);

#[derive(Default)]
struct UnnamedStructModel(i16, i8, f32);

#[derive(o2o)]
#[o2o(
    try_map(UnnamedStructDto as (), String),
    try_map(UnnamedStructModel as (), String),
    try_into_existing(UnnamedStructDto as (), String),
    try_into_existing(UnnamedStructModel as (), String),
)]
struct NamedStruct {
    #[try_from(UnnamedStructModel| @.0 as i32)]
    #[try_into(UnnamedStructModel| ~ as i16)]
    some_int: i32,
    #[from(UnnamedStructModel| @.1 as i32)]
    #[into(UnnamedStructModel| ~ as i8)]
    another_int: i32,
    some_float: f32,
}

#[derive(Default)]
struct NamedStructDto {
    some_int: i32,
    another_int: i32,
    some_float: f32,
}

#[derive(Default)]
struct NamedStructModel {
    some_int: i16,
    another_int: i8,
    some_float: f32,
}

#[derive(o2o)]
#[try_map(NamedStructDto as {}, String)]
#[try_map(NamedStructModel as {}, String)]
#[try_into_existing(NamedStructDto as {}, String)]
#[try_into_existing(NamedStructModel as {}, String)]
struct UnnamedStruct(
    #[o2o(
        map(NamedStructDto| some_int),
        from(NamedStructModel| @.some_int as i32),
        into(NamedStructModel| some_int, ~ as i16),
    )]
    i32, 
    #[o2o(map(NamedStructDto| another_int))]
    #[o2o(from(NamedStructModel| @.another_int as i32))]
    #[o2o(into(NamedStructModel| another_int, ~ as i8))]
    i32, 
    #[o2o(map(some_float))]
    f32
);

#[test]
fn named2unnamed() {
    let named = NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let dto: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(127, dto.1);
    assert_eq!(456.0, dto.2);

    let named = NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let model: UnnamedStructModel = named.try_into().unwrap();

    assert_eq!(123, model.0);
    assert_eq!(127, model.1);
    assert_eq!(456.0, model.2);
}

#[test]
fn named2unnamed_2() {
    let dto = NamedStructDto {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let unnamed: UnnamedStruct = dto.try_into().unwrap();

    assert_eq!(123, unnamed.0);
    assert_eq!(127, unnamed.1);
    assert_eq!(456.0, unnamed.2);

    let named = NamedStructModel {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let unnamed: UnnamedStruct = named.try_into().unwrap();

    assert_eq!(123, unnamed.0);
    assert_eq!(127, unnamed.1);
    assert_eq!(456.0, unnamed.2);
}

#[test]
fn unnamed2named() {
    let dto = UnnamedStructDto(123, 127, 456.0);

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(127, named.another_int);
    assert_eq!(456.0, named.some_float);

    let unnamed = UnnamedStructModel(123, 127, 456.0);

    let named: NamedStruct = unnamed.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(127, named.another_int);
    assert_eq!(456.0, named.some_float);
}

#[test]
fn unnamed2named_2() {
    let unnamed = UnnamedStruct(123, 127, 456.0);

    let dto: NamedStructDto = unnamed.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(127, dto.another_int);
    assert_eq!(456.0, dto.some_float);

    let unnamed = UnnamedStruct(123, 127, 456.0);

    let model: NamedStructModel = unnamed.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(127, model.another_int);
    assert_eq!(456.0, model.some_float);
}

#[test]
fn named2unnamed_ref() {
    let named = &NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let dto: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.0);
    assert_eq!(named.another_int, dto.1);
    assert_eq!(named.some_float, dto.2);

    let named = &NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let model: UnnamedStructModel = named.try_into().unwrap();

    assert_eq!(named.some_int as i16, model.0);
    assert_eq!(named.another_int as i8, model.1);
    assert_eq!(named.some_float, model.2);
}

#[test]
fn named2unnamed_2_ref() {
    let dto = &NamedStructDto {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let unnamed: UnnamedStruct = dto.try_into().unwrap();

    assert_eq!(dto.some_int, unnamed.0);
    assert_eq!(dto.another_int, unnamed.1);
    assert_eq!(dto.some_float, unnamed.2);

    let model = &NamedStructModel {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let unnamed: UnnamedStruct = model.try_into().unwrap();

    assert_eq!(model.some_int as i32, unnamed.0);
    assert_eq!(model.another_int as i32, unnamed.1);
    assert_eq!(model.some_float, unnamed.2);
}

#[test]
fn unnamed2named_ref() {
    let dto = &UnnamedStructDto(123, 127, 456.0);

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(dto.0, named.some_int);
    assert_eq!(dto.1, named.another_int);
    assert_eq!(dto.2, named.some_float);

    let model = &UnnamedStructModel(123, 127, 456.0);

    let named: NamedStruct = model.try_into().unwrap();

    assert_eq!(model.0 as i32, named.some_int);
    assert_eq!(model.1 as i32, named.another_int);
    assert_eq!(model.2, named.some_float);
}

#[test]
fn unnamed2named_2_ref() {
    let unnamed = &UnnamedStruct(123, 127, 456.0);

    let dto: NamedStructDto = unnamed.try_into().unwrap();

    assert_eq!(dto.some_int, dto.some_int);
    assert_eq!(dto.another_int, dto.another_int);
    assert_eq!(dto.some_float, dto.some_float);

    let unnamed = &UnnamedStruct(123, 127, 456.0);

    let model: NamedStructModel = unnamed.try_into().unwrap();

    assert_eq!(dto.some_int as i16, model.some_int);
    assert_eq!(dto.another_int as i8, model.another_int);
    assert_eq!(dto.some_float, model.some_float);
}

#[test]
fn existing_named2unnamed() {
    let named = NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let mut dto: UnnamedStructDto = Default::default();
    named.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(127, dto.1);
    assert_eq!(456.0, dto.2);

    let named = NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let mut model: UnnamedStructModel = Default::default();
    named.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.0);
    assert_eq!(127, model.1);
    assert_eq!(456.0, model.2);
}

#[test]
fn existing_unnamed2named() {
    let unnamed = UnnamedStruct(123, 127, 456.0);

    let mut dto: NamedStructDto = Default::default();
    unnamed.try_into_existing(&mut dto).unwrap();
    assert_eq!(123, dto.some_int);
    assert_eq!(127, dto.another_int);
    assert_eq!(456.0, dto.some_float);

    let unnamed = UnnamedStruct(123, 127, 456.0);

    let mut model: NamedStructModel = Default::default();
    unnamed.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(127, model.another_int);
    assert_eq!(456.0, model.some_float);
}

#[test]
fn existing_named2unnamed_ref() {
    let named = &NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let mut dto: UnnamedStructDto = Default::default();
    named.try_into_existing(&mut dto).unwrap();

    assert_eq!(named.some_int, dto.0);
    assert_eq!(named.another_int, dto.1);
    assert_eq!(named.some_float, dto.2);

    let named = &NamedStruct {
        some_int: 123,
        another_int: 127,
        some_float: 456.0
    };

    let mut model: UnnamedStructModel = Default::default();
    named.try_into_existing(&mut model).unwrap();

    assert_eq!(named.some_int as i16, model.0);
    assert_eq!(named.another_int as i8, model.1);
    assert_eq!(named.some_float, model.2);
}

#[test]
fn existing_unnamed2named_2_ref() {
    let unnamed = &UnnamedStruct(123, 127, 456.0);

    let mut dto: NamedStructDto = Default::default();
    unnamed.try_into_existing(&mut dto).unwrap();

    assert_eq!(dto.some_int, dto.some_int);
    assert_eq!(dto.another_int, dto.another_int);
    assert_eq!(dto.some_float, dto.some_float);

    let unnamed = &UnnamedStruct(123, 127, 456.0);

    let mut model: NamedStructModel = Default::default();
    unnamed.try_into_existing(&mut model).unwrap();

    assert_eq!(dto.some_int as i16, model.some_int);
    assert_eq!(dto.another_int as i8, model.another_int);
    assert_eq!(dto.some_float, model.some_float);
}