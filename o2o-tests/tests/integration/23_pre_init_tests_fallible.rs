use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(Default)]
struct Person {
    age: i8,
    first_name: String,
    last_name: String,
}

#[derive(o2o)]
#[o2o(try_from_owned(Person, String| vars(first_name: {@.first_name}, last_name: {@.last_name})))]
#[try_from_ref(Person, String| vars(first_name: {@.first_name.clone()}, last_name: {@.last_name.clone()}))]
#[try_into(Person, String| vars(first: {"John"}, last: {"Doe"}))]
#[try_into_existing(Person, String| vars(first: {"John"}, last: {"Doe"}))]
#[ghosts(first_name: {first.try_into().unwrap()}, last_name: {last.try_into().unwrap()})]
struct PersonDto {
    age: i8,
    #[ghost({format!("{} {}", first_name, last_name)})]
    full_name: String,
}

#[test]
fn named2named() {
    let person = Person {
        age: 42,
        first_name: "Dohn".try_into().unwrap(),
        last_name: "Joe".try_into().unwrap(),
    };

    let dto: PersonDto = person.try_into().unwrap();

    assert_eq!(42, dto.age);
    assert_eq!("Dohn Joe", dto.full_name);
}

#[test]
fn named2named_ref() {
    let person = &Person {
        age: 42,
        first_name: "Dohn".try_into().unwrap(),
        last_name: "Joe".try_into().unwrap(),
    };

    let dto: PersonDto = person.try_into().unwrap();

    assert_eq!(person.age, dto.age);
    assert_eq!(format!("{} {}", person.first_name, person.last_name), dto.full_name);
}

#[test]
fn named2named_reverse() {
    let dto = PersonDto { age: 42, full_name: "Test".try_into().unwrap() };

    let person: Person = dto.try_into().unwrap();

    assert_eq!(42, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}

#[test]
fn named2named_reverse_ref() {
    let dto = &PersonDto { age: 42, full_name: "Test".try_into().unwrap() };

    let person: Person = dto.try_into().unwrap();

    assert_eq!(dto.age, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}

#[test]
fn existing_named2named() {
    let dto = PersonDto { age: 42, full_name: "Test".try_into().unwrap() };

    let mut person: Person = Default::default();
    dto.try_into_existing(&mut person).unwrap();

    assert_eq!(42, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}

#[test]
fn existing_named2named_ref() {
    let dto = &PersonDto { age: 42, full_name: "Test".try_into().unwrap() };

    let mut person: Person = Default::default();
    dto.try_into_existing(&mut person).unwrap();

    assert_eq!(42, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}
