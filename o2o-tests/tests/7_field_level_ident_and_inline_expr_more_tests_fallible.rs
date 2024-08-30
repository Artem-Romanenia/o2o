use o2o::o2o;
use o2o::traits::TryIntoExisting;

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
#[try_map(Parent, anyhow::Error)]
#[try_map(ParentModel, anyhow::Error)]
#[try_into_existing(Parent, anyhow::Error)]
#[try_into_existing(ParentModel, anyhow::Error)]
struct ParentDto {
    #[from(Parent| (&@.child).try_into()?)]
    #[into(Parent| child, (&@.diff_child).try_into()?)]
    #[from(ParentModel| (&@.child_diff).try_into()?)]
    #[into(ParentModel| child_diff, (&@.diff_child).try_into()?)]
    diff_child: ChildDto,
    parent_int: i32,
}

#[derive(o2o)]
#[o2o(try_map(Child, anyhow::Error))]
struct ChildDto {
    #[o2o(from(@.child_int as i16))]
    #[o2o(into(@.child_int as i32))]
    child_int: i16,
    #[o2o(from(@.another_child_int as i8))]
    #[o2o(into(another_child_int, @.diff_another_child_int as i32))]
    diff_another_child_int: i8,
}

#[test]
fn named2named_different_name_and_type() {
    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto {
            child_int: 456,
            diff_another_child_int: 123,
        },
    };

    let p: Parent = dto.try_into().unwrap();

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto {
            child_int: 456,
            diff_another_child_int: 123,
        },
    };

    let model: ParentModel = dto.try_into().unwrap();

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn named2named_different_name_and_type_reverse() {
    let p = Parent {
        parent_int: 987,
        child: Child {
            child_int: 456,
            another_child_int: 123,
        },
    };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);

    let model = ParentModel {
        parent_int: 987,
        child_diff: Child {
            child_int: 456,
            another_child_int: 123,
        },
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
        diff_child: ChildDto {
            child_int: 456,
            diff_another_child_int: 123,
        },
    };

    let p: Parent = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, p.parent_int);
    assert_eq!(dto.diff_child.child_int, p.child.child_int as i16);
    assert_eq!(
        dto.diff_child.diff_another_child_int,
        p.child.another_child_int as i8
    );

    let model: ParentModel = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, model.parent_int);
    assert_eq!(dto.diff_child.child_int, model.child_diff.child_int as i16);
    assert_eq!(
        dto.diff_child.diff_another_child_int,
        model.child_diff.another_child_int as i8
    );
}

#[test]
fn named2named_different_name_and_type_reverse_ref() {
    let p = &Parent {
        parent_int: 987,
        child: Child {
            child_int: 456,
            another_child_int: 123,
        },
    };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(p.parent_int, dto.parent_int);
    assert_eq!(p.child.child_int, dto.diff_child.child_int as i32);
    assert_eq!(
        p.child.another_child_int,
        dto.diff_child.diff_another_child_int as i32
    );

    let model = &ParentModel {
        parent_int: 987,
        child_diff: Child {
            child_int: 456,
            another_child_int: 123,
        },
    };

    let dto: ParentDto = model.try_into().unwrap();

    assert_eq!(model.parent_int, dto.parent_int);
    assert_eq!(model.child_diff.child_int, dto.diff_child.child_int as i32);
    assert_eq!(
        model.child_diff.another_child_int,
        dto.diff_child.diff_another_child_int as i32
    );
}

#[test]
fn existing_named2named_different_name_and_type() {
    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto {
            child_int: 456,
            diff_another_child_int: 123,
        },
    };

    let mut p: Parent = Default::default();
    dto.try_into_existing(&mut p).unwrap();

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto {
        parent_int: 987,
        diff_child: ChildDto {
            child_int: 456,
            diff_another_child_int: 123,
        },
    };

    let mut model: ParentModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn existing_named2named_different_name_and_type_ref() {
    let dto = &ParentDto {
        parent_int: 987,
        diff_child: ChildDto {
            child_int: 456,
            diff_another_child_int: 123,
        },
    };

    let mut p: Parent = Default::default();
    dto.try_into_existing(&mut p).unwrap();

    assert_eq!(dto.parent_int, p.parent_int);
    assert_eq!(dto.diff_child.child_int, p.child.child_int as i16);
    assert_eq!(
        dto.diff_child.diff_another_child_int,
        p.child.another_child_int as i8
    );

    let mut model: ParentModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(dto.parent_int, model.parent_int);
    assert_eq!(dto.diff_child.child_int, model.child_diff.child_int as i16);
    assert_eq!(
        dto.diff_child.diff_another_child_int,
        model.child_diff.another_child_int as i8
    );
}
