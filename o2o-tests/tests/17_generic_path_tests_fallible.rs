use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(Default)]
struct Parent<T> {
    child: Child<i16>,
    parent_int: T,
}

#[derive(Default)]
struct ParentModel<T> {
    child_diff: Child<i32>,
    parent_int: T,
}

#[derive(Default)]
struct Child<T> {
    child_int: T,
    another_child_int: T,
}

#[derive(o2o)]
#[o2o(
    try_map(Parent::<i32>, String),
    try_map(ParentModel::<i32>, String),
    try_into_existing(Parent::<i32>, String),
    try_into_existing(ParentModel::<i32>, String),
)]
struct ParentDto {
    parent_int: i32,
    #[o2o(
        from(Parent::<i32>| (&@.child).try_into().unwrap()),
        into(Parent::<i32>| child, (&@.diff_child).try_into().unwrap()),
        from(ParentModel::<i32>| (&@.child_diff).try_into().unwrap()),
        into(ParentModel::<i32>| child_diff, (&@.diff_child).try_into().unwrap()),
    )]
    diff_child: ChildDto,
}

#[derive(o2o)]
#[try_map(Child::<i32>, String)]
#[try_map(Child::<i16>, String)]
struct ChildDto {
    #[o2o(from(Child::<i32>| @.child_int as i16))]
    #[o2o(into(Child::<i32>| @.child_int as i32))]
    child_int: i16,
    #[from(Child::<i32>| @.another_child_int as i8)]
    #[into(Child::<i32>| another_child_int, @.diff_another_child_int as i32)]
    #[from(Child::<i16>| @.another_child_int as i8)]
    #[into(Child::<i16>| another_child_int, @.diff_another_child_int as i16)]
    diff_another_child_int: i8,
}

#[test]
fn named2named_different_name_and_type() {
    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let p: Parent<i32> = dto.try_into().unwrap();

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let model: ParentModel<i32> = dto.try_into().unwrap();

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn named2named_different_name_and_type_reverse() {
    let p = Parent { parent_int: 987, child: Child { child_int: 456, another_child_int: 123 } };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);

    let model = ParentModel {
        parent_int: 987,
        child_diff: Child { child_int: 456, another_child_int: 123 },
    };

    let dto: ParentDto = model.try_into().unwrap();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);
}

#[test]
fn named2named_different_name_and_type_ref() {
    let dto = &ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let p: Parent<i32> = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, p.parent_int);
    assert_eq!(dto.diff_child.child_int, p.child.child_int as i16);
    assert_eq!(dto.diff_child.diff_another_child_int, p.child.another_child_int as i8);

    let model: ParentModel<i32> = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, model.parent_int);
    assert_eq!(dto.diff_child.child_int, model.child_diff.child_int as i16);
    assert_eq!(dto.diff_child.diff_another_child_int, model.child_diff.another_child_int as i8);
}

#[test]
fn named2named_different_name_and_type_reverse_ref() {
    let p = &Parent { parent_int: 987, child: Child { child_int: 456, another_child_int: 123 } };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(p.parent_int, dto.parent_int);
    assert_eq!(p.child.child_int, dto.diff_child.child_int);
    assert_eq!(p.child.another_child_int, dto.diff_child.diff_another_child_int as i16);

    let model = &ParentModel {
        parent_int: 987,
        child_diff: Child { child_int: 456, another_child_int: 123 },
    };

    let dto: ParentDto = model.try_into().unwrap();

    assert_eq!(model.parent_int, dto.parent_int);
    assert_eq!(model.child_diff.child_int, dto.diff_child.child_int as i32);
    assert_eq!(model.child_diff.another_child_int, dto.diff_child.diff_another_child_int as i32);
}

#[test]
fn existing_named2named_different_name_and_type() {
    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let mut p: Parent<i32> = Default::default();
    dto.try_into_existing(&mut p).unwrap();

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let mut model: ParentModel<i32> = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn existing_named2named_different_name_and_type_ref() {
    let dto = &ParentDto {
        parent_int: 987,
        diff_child: ChildDto { child_int: 456, diff_another_child_int: 123 },
    };

    let mut p: Parent<i32> = Default::default();
    dto.try_into_existing(&mut p).unwrap();

    assert_eq!(dto.parent_int, p.parent_int);
    assert_eq!(dto.diff_child.child_int, p.child.child_int as i16);
    assert_eq!(dto.diff_child.diff_another_child_int, p.child.another_child_int as i8);

    let mut model: ParentModel<i32> = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(dto.parent_int, model.parent_int);
    assert_eq!(dto.diff_child.child_int, model.child_diff.child_int as i16);
    assert_eq!(dto.diff_child.diff_another_child_int, model.child_diff.another_child_int as i8);
}
