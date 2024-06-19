use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(Default)]
struct Parent {
    child: Child,
    parent_int: i32,
}

#[derive(Default)]
struct Child {
    child_int: i32,
    another_child_int: i32,
}

#[derive(o2o)]
#[try_map(Parent, anyhow::Error)]
#[try_into_existing(Parent, anyhow::Error)]
struct ParentDto {
    #[map((&@.child).try_into()?)]
    child: ChildDto,
    parent_int: i32,
}

#[derive(o2o)]
#[try_map(Child, anyhow::Error)]
struct ChildDto {
    child_int: i32,
    #[map(another_child_int)]
    diff_another_child_int: i32,
}

#[test]
fn named2named_child() {
    let p = Parent {
        parent_int: 123,
        child: Child { 
            child_int: 321, 
            another_child_int: 456, 
        },
    };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.child.child_int);
    assert_eq!(456, dto.child.diff_another_child_int);
}

#[test]
fn named2named_child_reverse() {
    let dto = ParentDto {
        parent_int: 123,
        child: ChildDto { 
            child_int: 321, 
            diff_another_child_int: 456, 
        },
    };

    let parent: Parent = dto.try_into().unwrap();

    assert_eq!(123, parent.parent_int);
    assert_eq!(321, parent.child.child_int);
    assert_eq!(456, parent.child.another_child_int);
}

#[test]
fn named2named_child_ref() {
    let p = &Parent {
        parent_int: 123,
        child: Child { 
            child_int: 321, 
            another_child_int: 456, 
        },
    };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(p.parent_int, dto.parent_int);
    assert_eq!(p.child.child_int, dto.child.child_int);
    assert_eq!(p.child.another_child_int, dto.child.diff_another_child_int);
}

#[test]
fn named2named_child_ref_reverse() {
    let dto = &ParentDto {
        parent_int: 123,
        child: ChildDto { 
            child_int: 321, 
            diff_another_child_int: 456, 
        },
    };

    let parent: Parent = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, parent.parent_int);
    assert_eq!(dto.child.child_int, parent.child.child_int);
    assert_eq!(dto.child.diff_another_child_int, parent.child.another_child_int);
}

#[test]
fn existing_named2named_child_reverse() {
    let dto = ParentDto {
        parent_int: 123,
        child: ChildDto { 
            child_int: 321, 
            diff_another_child_int: 456, 
        },
    };

    let mut parent: Parent = Default::default();
    dto.try_into_existing(&mut parent).unwrap();

    assert_eq!(123, parent.parent_int);
    assert_eq!(321, parent.child.child_int);
    assert_eq!(456, parent.child.another_child_int);
}

#[test]
fn existing_named2named_child_ref_reverse() {
    let dto = &ParentDto {
        parent_int: 123,
        child: ChildDto { 
            child_int: 321, 
            diff_another_child_int: 456, 
        },
    };

    let mut parent: Parent = Default::default();
    dto.try_into_existing(&mut parent).unwrap();

    assert_eq!(dto.parent_int, parent.parent_int);
    assert_eq!(dto.child.child_int, parent.child.child_int);
    assert_eq!(dto.child.diff_another_child_int, parent.child.another_child_int);
}