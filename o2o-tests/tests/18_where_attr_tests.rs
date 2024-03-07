use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct Parent<TChild,T> {
    child: TChild,
    parent_int: T,
}

#[derive(Default)]
struct ParentModel<T> {
    child_diff: Child<T>,
    parent_int: i32,
}

#[derive(Default)]
struct Child<T> {
    child_int: T,
    another_child_int: T,
}

#[derive(o2o)]
#[o2o(
    map(Parent::<Child::<T>, i32>),
    map(ParentModel::<T>),
    into_existing(Parent::<Child::<T>, i32>),
    into_existing(ParentModel::<T>),
    where_clause(T: Copy),
)]
struct ParentDto<T> where T: Copy {
    parent_int: i32,
    #[o2o(
        from(Parent::<Child::<T>, i32>| (&@.child).into()),
        owned_into(Parent::<Child::<T>, i32>| child, ~.into()),
        ref_into(Parent::<Child::<T>, i32>| child, (&@.diff_child).into()),
        from_owned(ParentModel::<T>| @.child_diff.into()),
    )]
    #[o2o(from_ref(ParentModel::<T>| (&@.child_diff).into()))]
    #[into(ParentModel::<T>| child_diff, (&@.diff_child).into())]
    diff_child: ChildDto<T>,
}

#[derive(o2o)]
#[map(Child::<T>)]
#[where_clause(T: Copy)]
struct ChildDto<T> where T: Copy {
    child_int: T,
    #[map(another_child_int)]
    diff_another_child_int: T,
}

#[test]
fn named2named_different_name_and_type() {
    let dto = ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let p: Parent::<Child<i16>, i32> = dto.into();

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let model: ParentModel::<i32> = dto.into();

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn named2named_different_name_and_type_reverse() {
    let p = Parent{
        parent_int: 987,
        child: Child { 
            child_int: 456, 
            another_child_int: 123
        }
    };

    let dto: ParentDto<i32> = p.into();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);

    let model = ParentModel{
        parent_int: 987,
        child_diff: Child { 
            child_int: 456, 
            another_child_int: 123
        }
    };

    let dto: ParentDto<i32> = model.into();

    assert_eq!(987, dto.parent_int);
    assert_eq!(456, dto.diff_child.child_int);
    assert_eq!(123, dto.diff_child.diff_another_child_int);
}

#[test]
fn named2named_different_name_and_type_ref() {
    let dto = &ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let p: Parent::<Child<i16>, i32> = dto.into();

    assert_eq!(dto.parent_int, p.parent_int);
    assert_eq!(dto.diff_child.child_int, p.child.child_int as i16);
    assert_eq!(dto.diff_child.diff_another_child_int, p.child.another_child_int);

    let model: ParentModel::<i16> = dto.into();

    assert_eq!(dto.parent_int, model.parent_int);
    assert_eq!(dto.diff_child.child_int, model.child_diff.child_int as i16);
    assert_eq!(dto.diff_child.diff_another_child_int, model.child_diff.another_child_int as i16);
}

#[test]
fn named2named_different_name_and_type_reverse_ref() {
    let p = &Parent{
        parent_int: 987,
        child: Child { 
            child_int: 456, 
            another_child_int: 123
        }
    };

    let dto: ParentDto<i32> = p.into();

    assert_eq!(p.parent_int, dto.parent_int);
    assert_eq!(p.child.child_int, dto.diff_child.child_int);
    assert_eq!(p.child.another_child_int, dto.diff_child.diff_another_child_int);

    let model = &ParentModel{
        parent_int: 987,
        child_diff: Child { 
            child_int: 456, 
            another_child_int: 123
        }
    };

    let dto: ParentDto<i32> = model.into();

    assert_eq!(model.parent_int, dto.parent_int);
    assert_eq!(model.child_diff.child_int, dto.diff_child.child_int);
    assert_eq!(model.child_diff.another_child_int, dto.diff_child.diff_another_child_int);
}

#[test]
fn existing_named2named_different_name_and_type() {
    let dto = ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let mut p: Parent::<Child<i16>, i32> = Default::default();
    dto.into_existing(&mut p);

    assert_eq!(987, p.parent_int);
    assert_eq!(456, p.child.child_int);
    assert_eq!(123, p.child.another_child_int);

    let dto = ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let mut model: ParentModel::<i32> = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(987, model.parent_int);
    assert_eq!(456, model.child_diff.child_int);
    assert_eq!(123, model.child_diff.another_child_int);
}

#[test]
fn existing_named2named_different_name_and_type_ref() {
    let dto = &ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let mut p: Parent::<Child<i16>, i32> = Default::default();
    dto.into_existing(&mut p);

    assert_eq!(dto.parent_int, p.parent_int);
    assert_eq!(dto.diff_child.child_int, p.child.child_int);
    assert_eq!(dto.diff_child.diff_another_child_int, p.child.another_child_int);

    let dto = &ParentDto{
        parent_int: 987,
        diff_child: ChildDto { 
            child_int: 456, 
            diff_another_child_int: 123
        }
    };

    let mut model: ParentModel::<i32> = Default::default();
    dto.into_existing(&mut model);

    assert_eq!(dto.parent_int, model.parent_int);
    assert_eq!(dto.diff_child.child_int, model.child_diff.child_int);
    assert_eq!(dto.diff_child.diff_another_child_int, model.child_diff.another_child_int);
}