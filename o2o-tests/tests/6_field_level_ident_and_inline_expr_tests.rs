use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct Parent {
    child: Child,
    parent_int: i32,
}

#[derive(Default)]
struct ParentModel {
    child_diff: Child,
    parent_int: i32,
}

#[derive(Default)]
struct Child {
    child_int: i32,
    another_child_int: i32,
}

#[derive(o2o)]
#[o2o(map_owned(Parent), map_owned(ParentModel), owned_into_existing(Parent), owned_into_existing(ParentModel))]
struct ParentDto {
    #[o2o(
        from_owned(Parent| @.child.into()),
        owned_into(Parent| child, ~.into()),
        from_owned(ParentModel| @.child_diff.into()),
        owned_into(ParentModel| child_diff, @.diff_child.into()),
    )]
    diff_child: ChildDto,
    parent_int: i32,
}

#[derive(o2o)]
#[map(Child)]
struct ChildDto {
    #[from(~ as i16)]
    #[into(~ as i32)]
    child_int: i16,
    #[from(@.another_child_int as i8)]
    #[into(another_child_int, ~ as i32)]
    diff_another_child_int: i8,
}

#[test]
fn named2named_different_name_and_type() {
    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let p: Parent = dto.into();

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let model: ParentModel = dto.into();

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn named2named_different_name_and_type_reverse() {
    let p = Parent {
        parent_int: 987,
        child: Child { child_int: 456, another_child_int: 123 },
    };

    let dto: ParentDto = p.into();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);

    let model = ParentModel {
        parent_int: 987,
        child_diff: Child { child_int: 456, another_child_int: 123 },
    };

    let dto: ParentDto = model.into();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);
}

#[test]
fn existing_named2named_different_name_and_type() {
    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let mut p: Parent = Default::default();
    dto.into_existing(&mut p);

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let mut model: ParentModel = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}
