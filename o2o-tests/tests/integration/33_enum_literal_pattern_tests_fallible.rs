use test_case::test_case;

#[derive(PartialEq, Debug, o2o::o2o)]
#[try_map(i32, String)]
enum HttpStatus {
    #[literal(200)]
    Ok,
    #[literal(404)]
    NotFound,
    #[literal(500)]
    InternalError,
    #[pattern(_)]
    #[owned_into({f0})]
    #[ref_into({*f0})]
    Other(
        #[from_owned(@)]
        #[from_ref(*@)]
        i32,
    ),
}

type StaticStr = &'static str;

#[derive(PartialEq, Debug, o2o::o2o)]
#[try_map_owned(StaticStr, String)]
enum Animal {
    #[literal("🐶")]
    Dog,
    #[literal("🐱")]
    Cat,
    #[literal("🐵")]
    Monkey,
    #[pattern(_)]
    #[into({name})]
    Other {
        #[from(@)]
        name: StaticStr,
    },
}

#[test_case(200, HttpStatus::Ok ; "200_OK")]
#[test_case(404, HttpStatus::NotFound ; "400_NotFound")]
#[test_case(500, HttpStatus::InternalError ; "500_InternalError")]
#[test_case(204, HttpStatus::Other(204) ; "204_Other")]
fn http_status_success(lit: i32, status: HttpStatus) {
    let s: HttpStatus = lit.try_into().unwrap();
    assert_eq!(status, s);

    let l: i32 = status.try_into().unwrap();
    assert_eq!(lit, l);
}

#[test_case(200, HttpStatus::Ok ; "200_OK")]
#[test_case(404, HttpStatus::NotFound ; "400_NotFound")]
#[test_case(500, HttpStatus::InternalError ; "500_InternalError")]
#[test_case(204, HttpStatus::Other(204) ; "204_Other")]
fn http_status_ref_success(lit: i32, status: HttpStatus) {
    let lit_ref = &lit;
    let status_ref = &status;

    let s: HttpStatus = lit_ref.try_into().unwrap();
    assert_eq!(status, s);

    let l: i32 = status_ref.try_into().unwrap();
    assert_eq!(lit, l);
}

#[test_case("🐶", Animal::Dog ; "Dog")]
#[test_case("🐱", Animal::Cat ; "Cat")]
#[test_case("🐵", Animal::Monkey ; "Monkey")]
#[test_case("Whale", Animal::Other { name: "Whale" } ; "Whale")]
fn animal_success(lit: &'static str, animal: Animal) {
    let a: Animal = lit.try_into().unwrap();
    assert_eq!(animal, a);

    let l: &str = animal.try_into().unwrap();
    assert_eq!(lit, l);
}
