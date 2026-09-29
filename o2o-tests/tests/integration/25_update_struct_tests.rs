use o2o::o2o;

struct Entity {
    some_int: i32,
    some_float: f32,
}

impl Default for Entity {
    fn default() -> Self {
        Self { some_int: 0, some_float: 321.0 }
    }
}

#[derive(o2o)]
#[into(Entity| ..Default::default())]
#[from(Entity| ..get_default())]
struct EntityDto {
    some_int: i32,
    #[ghost]
    some_string: String,
}

fn get_default() -> EntityDto {
    EntityDto { some_int: 0, some_string: "test".into() }
}

#[test]
fn named2named() {
    let dto = EntityDto { some_int: 123, some_string: "321".into() };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.some_int);
    assert_eq!(321.0, entity.some_float);
}

#[test]
fn named2named_reverse() {
    let entity = Entity { some_int: 123, some_float: 654.0 };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("test", dto.some_string);
}

#[test]
fn named2named_ref() {
    let dto = &EntityDto { some_int: 123, some_string: "321".into() };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.some_int);
    assert_eq!(321.0, entity.some_float);
}

#[test]
fn named2named_ref_reverse() {
    let entity = &Entity { some_int: 123, some_float: 654.0 };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("test", dto.some_string);
}
