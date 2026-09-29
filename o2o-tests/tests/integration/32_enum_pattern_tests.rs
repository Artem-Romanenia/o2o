use test_case::{test_case, test_matrix};

#[derive(PartialEq, Debug, o2o::o2o)]
#[from(i32| _ => panic!())]
enum HttpStatusFamily {
    #[pattern(100..=199)]
    Information,
    #[pattern(200..=299)]
    Success,
    #[pattern(300..=399)]
    Redirection,
    #[pattern(400..=499)]
    ClientError,
    #[pattern(500..=599)]
    ServerError,
}

type StaticStr = &'static str;

#[derive(PartialEq, Debug, o2o::o2o)]
#[from_owned(StaticStr| _ => todo!())]
enum AnimalKind {
    #[pattern("🐶" | "🐱" | "🐵")]
    Mammal,

    #[pattern("🐟")]
    Fish,

    #[pattern("🐛" | "🐜")]
    Insect,
}

#[test_matrix([101, 102], [HttpStatusFamily::Information] ; "100_Info")]
#[test_matrix([200, 204], [HttpStatusFamily::Success] ; "200_Success")]
#[test_matrix([301, 303], [HttpStatusFamily::Redirection] ; "300_Redirection")]
#[test_matrix([400, 404], [HttpStatusFamily::ClientError] ; "400_ClientError")]
#[test_matrix([500, 501], [HttpStatusFamily::ServerError] ; "500_ServerError")]
fn http_status_success(lit: i32, status: HttpStatusFamily) {
    let s: HttpStatusFamily = lit.into();
    assert_eq!(status, s);
}

#[test_matrix([101, 102], [HttpStatusFamily::Information] ; "100_Info")]
#[test_matrix([200, 204], [HttpStatusFamily::Success] ; "200_Success")]
#[test_matrix([301, 303], [HttpStatusFamily::Redirection] ; "300_Redirection")]
#[test_matrix([400, 404], [HttpStatusFamily::ClientError] ; "400_ClientError")]
#[test_matrix([500, 501], [HttpStatusFamily::ServerError] ; "500_ServerError")]
fn http_status_ref_success(lit: i32, status: HttpStatusFamily) {
    let lit_ref = &lit;

    let s: HttpStatusFamily = lit_ref.into();
    assert_eq!(status, s);
}

#[test_case("🐶", AnimalKind::Mammal ; "Dog")]
#[test_case("🐱", AnimalKind::Mammal ; "Cat")]
#[test_case("🐵", AnimalKind::Mammal ; "Monkey")]
#[test_case("🐟", AnimalKind::Fish ; "Fish")]
#[test_case("🐛", AnimalKind::Insect ; "Caterpillar")]
#[test_case("🐜", AnimalKind::Insect ; "Bug")]
fn animal_success(lit: &'static str, animal: AnimalKind) {
    let a: AnimalKind = lit.into();
    assert_eq!(animal, a);
}

#[test]
#[should_panic = "explicit panic"]
fn http_status_ref_failure() {
    let lit = &999;

    let _ = HttpStatusFamily::from(lit);
}

#[test]
#[should_panic = "not yet implemented"]
fn animal_failure() {
    let a = "Whale";

    let _ = AnimalKind::from(a);
}
