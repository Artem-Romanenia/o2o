use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(o2o)]
#[try_map((i32, String), String)]
#[try_into_existing((i32, String), String)]
pub struct Entity {
    #[map(0)]
    int: i32,
    #[map_owned(1)]
    #[map_ref(1, ~.clone())]
    string: String,
}

#[test]
fn named2nameless() {
    let entity = Entity { int: 123, string: "Test".try_into().unwrap() };

    let (int, string) = entity.try_into().unwrap();

    assert_eq!(123, int);
    assert_eq!("Test", string);
}

#[test]
fn named2nameless_ref() {
    let entity = &Entity { int: 123, string: "Test".try_into().unwrap() };

    let (int, string) = entity.try_into().unwrap();

    assert_eq!(entity.int, int);
    assert_eq!(entity.string, string);
}

#[test]
fn named2nameless_reverse() {
    let tpl = (123, String::from("Test"));

    let entity: Entity = tpl.try_into().unwrap();

    assert_eq!(123, entity.int);
    assert_eq!("Test", entity.string);
}

#[test]
fn named2nameless_reverse_ref() {
    let tpl = &(123, String::from("Test"));

    let entity: Entity = tpl.try_into().unwrap();

    assert_eq!(tpl.0, entity.int);
    assert_eq!(tpl.1, entity.string);
}

#[test]
fn existing_named2nameless() {
    let entity = Entity { int: 123, string: "Test".try_into().unwrap() };

    let mut tpl = <(i32, String)>::default();
    entity.try_into_existing(&mut tpl).unwrap();

    assert_eq!(123, tpl.0);
    assert_eq!("Test", tpl.1);
}

#[test]
fn existing_named2nameless_ref() {
    let entity = &Entity { int: 123, string: "Test".try_into().unwrap() };

    let mut tpl = <(i32, String)>::default();
    entity.try_into_existing(&mut tpl).unwrap();

    assert_eq!(entity.int, tpl.0);
    assert_eq!(entity.string, tpl.1);
}
