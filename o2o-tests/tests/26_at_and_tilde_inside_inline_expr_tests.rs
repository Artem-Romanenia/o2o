use o2o::{o2o, traits::IntoExisting};

#[derive(Default)]
struct Entity {
    some_int: i32,
    another_int: i16,
    some_float: f32,
    another_float: f64,
    extra_float: f32,
    child: Child,
}

#[derive(Default)]
struct Child {
    child_float: f32,
}

fn another_int_to_string(e: &Entity) -> String {
    e.another_int.to_string()
}

fn another_int_string_to_int(f: &EntityDto) -> i16 {
    f.another_int_string.parse().unwrap()
}

fn unnamed_another_int_string_to_int(f: &UnnamedEntityDto) -> i16 {
    f.1.parse().unwrap()
}

fn float_to_string(f: f32) -> String {
    f.to_string()
}

fn string_to_float(f: String) -> f32 {
    f.parse().unwrap()
}

fn float64_to_string(f: f64) -> String {
    f.to_string()
}

fn string_to_float64(f: String) -> f64 {
    f.parse().unwrap()
}

fn extra_string(e: &Entity) -> String {
    e.extra_float.to_string()
}

fn extra_float(e: &EntityDto) -> f32 {
    e.extra_string.parse().unwrap()
}

fn extra_float_2(e: &UnnamedEntityDto) -> f32 {
    e.4.parse().unwrap()
}

#[derive(o2o)]
#[map(Entity)]
#[into_existing(Entity)]
#[child_parents(child: Child)]
#[ghosts(extra_float: extra_float(&@))]
struct EntityDto {
    some_int: i32,

    #[o2o(
        from_ref({ another_int_to_string(@) } ),
        from_owned({ another_int_to_string(&@) } ),
        ref_into(another_int, another_int_string_to_int(@)),
        owned_into(another_int, another_int_string_to_int(&@))
    )]
    another_int_string: String,

    #[o2o(from({ float_to_string(~) } ))]
    #[o2o(ref_into({ string_to_float(~.clone()) } ))]
    #[o2o(owned_into({ string_to_float(~.clone()) } ))]
    some_float: String,

    #[from(another_float, { float64_to_string(~) } )]
    #[ref_into(another_float, { string_to_float64(~.clone()) } )]
    #[owned_into(another_float, { string_to_float64(~.clone()) } )]
    another_float_string: String,

    #[ghost(extra_string(&@))]
    extra_string: String,

    #[child(child)]
    #[from(child_float, { float_to_string(~) })]
    #[into(child_float, string_to_float(~.clone()))]
    child_float_string: String,
}

#[derive(o2o)]
#[map(Entity as {})]
#[into_existing(Entity as {})]
#[child_parents(child: Child as {})]
#[ghosts(extra_float: { extra_float_2(&@) })]
struct UnnamedEntityDto(
    #[map(Entity| some_int)] i32,
    #[o2o(
        from_ref(another_int_to_string(@)),
        from_owned(another_int_to_string(&@)),
        ref_into(another_int, { unnamed_another_int_string_to_int(@) } ),
        owned_into(another_int, { unnamed_another_int_string_to_int(&@) } )
    )]
    String,
    #[o2o(from(some_float, { float_to_string(~) } ))]
    #[o2o(ref_into(some_float, { string_to_float(~.clone()) } ))]
    #[o2o(owned_into(some_float, { string_to_float(~.clone()) } ))]
    String,
    #[from(another_float, { float64_to_string(~) } )]
    #[ref_into(another_float, { string_to_float64(~.clone()) } )]
    #[owned_into(another_float, { string_to_float64(~.clone()) } )]
    String,
    #[ghost({ extra_string(&@) })] String,
    #[child(child)]
    #[from(child_float, { float_to_string(~) })]
    #[into(child_float, { string_to_float(~.clone()) })]
    String,
);

#[test]
fn named2named() {
    let dto = EntityDto {
        some_int: 123,
        another_int_string: "456".into(),
        some_float: "789".into(),
        another_float_string: "987".into(),
        extra_string: "654".into(),
        child_float_string: "321".into(),
    };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn named2named_reverse() {
    let entity = Entity {
        some_int: 123,
        another_int: 456,
        some_float: 789.0,
        another_float: 987.0,
        extra_float: 654.0,
        child: Child { child_float: 321.0 },
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("456", dto.another_int_string);
    assert_eq!("789", dto.some_float);
    assert_eq!("987", dto.another_float_string);
    assert_eq!("654", dto.extra_string);
    assert_eq!("321", dto.child_float_string);
}

#[test]
fn named2named_ref() {
    let dto = &EntityDto {
        some_int: 123,
        another_int_string: "456".into(),
        some_float: "789".into(),
        another_float_string: "987".into(),
        extra_string: "654".into(),
        child_float_string: "321".into(),
    };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn named2named_ref_reverse() {
    let entity = &Entity {
        some_int: 123,
        another_int: 456,
        some_float: 789.0,
        another_float: 987.0,
        extra_float: 654.0,
        child: Child { child_float: 321.0 },
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("456", dto.another_int_string);
    assert_eq!("789", dto.some_float);
    assert_eq!("987", dto.another_float_string);
    assert_eq!("654", dto.extra_string);
    assert_eq!("321", dto.child_float_string);
}

#[test]
fn existing_named2named() {
    let dto = EntityDto {
        some_int: 123,
        another_int_string: "456".into(),
        some_float: "789".into(),
        another_float_string: "987".into(),
        extra_string: "654".into(),
        child_float_string: "321".into(),
    };

    let mut entity: Entity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn existing_named2named_ref() {
    let dto = &EntityDto {
        some_int: 123,
        another_int_string: "456".into(),
        some_float: "789".into(),
        another_float_string: "987".into(),
        extra_string: "654".into(),
        child_float_string: "321".into(),
    };

    let mut entity: Entity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn unnamed2named() {
    let dto = UnnamedEntityDto(123, "456".into(), "789".into(), "987".into(), "654".into(), "321".into());

    let entity: Entity = dto.into();

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn unnamed2named_reverse() {
    let entity = Entity {
        some_int: 123,
        another_int: 456,
        some_float: 789.0,
        another_float: 987.0,
        extra_float: 654.0,
        child: Child { child_float: 321.0 },
    };

    let dto: UnnamedEntityDto = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!("456", dto.1);
    assert_eq!("789", dto.2);
    assert_eq!("987", dto.3);
    assert_eq!("654", dto.4);
    assert_eq!("321", dto.5);
}

#[test]
fn unnamed2named_ref() {
    let dto = &UnnamedEntityDto(123, "456".into(), "789".into(), "987".into(), "654".into(), "321".into());

    let entity: Entity = dto.into();

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn unnamed2named_ref_reverse() {
    let entity = &Entity {
        some_int: 123,
        another_int: 456,
        some_float: 789.0,
        another_float: 987.0,
        extra_float: 654.0,
        child: Child { child_float: 321.0 },
    };

    let dto: UnnamedEntityDto = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!("456", dto.1);
    assert_eq!("789", dto.2);
    assert_eq!("987", dto.3);
    assert_eq!("654", dto.4);
    assert_eq!("321", dto.5);
}

#[test]
fn existing_unnamed2named() {
    let dto = UnnamedEntityDto(123, "456".into(), "789".into(), "987".into(), "654".into(), "321".into());

    let mut entity: Entity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}

#[test]
fn existing_unnamed2named_ref() {
    let dto = &UnnamedEntityDto(123, "456".into(), "789".into(), "987".into(), "654".into(), "321".into());

    let mut entity: Entity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.some_int);
    assert_eq!(456, entity.another_int);
    assert_eq!(789.0, entity.some_float);
    assert_eq!(987.0, entity.another_float);
    assert_eq!(654.0, entity.extra_float);
    assert_eq!(321.0, entity.child.child_float);
}
