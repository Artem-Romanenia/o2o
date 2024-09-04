use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct Person {
    age: i8,
    first_name: String,
    last_name: String,
}

#[derive(o2o)]
#[o2o(from_owned(Person| vars(first_name: {@.first_name}, last_name: {@.last_name})))]
#[from_ref(Person| vars(first_name: {@.first_name.clone()}, last_name: {@.last_name.clone()}))]
#[into(Person| vars(first: {"John"}, last: {"Doe"}))]
#[into_existing(Person| vars(first: {"John"}, last: {"Doe"}))]
#[ghosts(first_name: {first.into()}, last_name: {last.into()})]
struct PersonDto {
    age: i8,
    #[ghost({format!("{} {}", first_name, last_name)})]
    full_name: String,
}

#[test]
fn named2named() {
    let person = Person { age: 42, first_name: "Dohn".into(), last_name: "Joe".into() };

    let dto: PersonDto = person.into();

    assert_eq!(42, dto.age);
    assert_eq!("Dohn Joe", dto.full_name);
}

#[test]
fn named2named_ref() {
    let person = &Person { age: 42, first_name: "Dohn".into(), last_name: "Joe".into() };

    let dto: PersonDto = person.into();

    assert_eq!(person.age, dto.age);
    assert_eq!(format!("{} {}", person.first_name, person.last_name), dto.full_name);
}

#[test]
fn named2named_reverse() {
    let dto = PersonDto { age: 42, full_name: "Test".into() };

    let person: Person = dto.into();

    assert_eq!(42, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}

#[test]
fn named2named_reverse_ref() {
    let dto = &PersonDto { age: 42, full_name: "Test".into() };

    let person: Person = dto.into();

    assert_eq!(dto.age, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}

#[test]
fn existing_named2named() {
    let dto = PersonDto { age: 42, full_name: "Test".into() };

    let mut person: Person = Default::default();
    dto.into_existing(&mut person);

    assert_eq!(42, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}

#[test]
fn existing_named2named_ref() {
    let dto = &PersonDto { age: 42, full_name: "Test".into() };

    let mut person: Person = Default::default();
    dto.into_existing(&mut person);

    assert_eq!(42, person.age);
    assert_eq!("John", person.first_name);
    assert_eq!("Doe", person.last_name);
}
