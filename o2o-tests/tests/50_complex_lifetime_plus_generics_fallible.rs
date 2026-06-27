use o2o::traits::TryIntoExisting;

#[derive(o2o::o2o)]
#[try_map(EntityDto<'a, 'b, N, T>, anyhow::Error)]
#[try_into_existing(EntityDto<'a, 'b, N, T>, anyhow::Error)]
#[try_map(TupleEntityDto<'a, 'b, N, T> as (), anyhow::Error)]
#[try_into_existing(TupleEntityDto<'a, 'b, N, T> as (), anyhow::Error)]
struct Entity<'a, 'b : 'a, const N: usize = 5, T : Copy = i32, const M : usize = 10> {
    parent_int: T,
    #[parent(EntityDto<'a, 'b, N, T>| [parent([map(base_int)] base_int_2, another_base_int)] base: Base, base_entity_int, base_string)]
    #[parent(TupleEntityDto<'a, 'b, N, T>| [parent([map(1)] base_int_2, [map(2)]another_base_int)] base: Base, [map(3)]base_entity_int, [map(4)] base_string)]
    base: BaseEntity<'a, T>,
    #[parent(EntityDto<'a, 'b, N, T>| child_int, another_child_int, child_string)]
    #[parent(TupleEntityDto<'a, 'b, N, T>| [map(5)] child_int, [map(6)] another_child_int, [map(7)] child_string)]
    child: Child<'b>
}

#[derive(o2o::o2o)]
#[try_map(TupleEntityDto<'a, 'b, N, T>, anyhow::Error)]
#[try_into_existing(TupleEntityDto<'a, 'b, N, T>, anyhow::Error)]
#[try_map(EntityDto<'a, 'b, N, T> as {}, anyhow::Error)]
#[try_into_existing(EntityDto<'a, 'b, N, T> as {}, anyhow::Error)]
struct TupleEntity<'a, 'b : 'a, const N: usize = 10, T : Copy = i32, const M : usize = 10>(
    #[map(EntityDto<'a, 'b, N, T>| parent_int)] T,

    #[parent(TupleEntityDto<'a, 'b, N, T>| [parent([map(1)] 0, [map(2)] 1,)] 0: TupleBase, [map(3)] 1, [map(4)] 2)]
    #[parent(EntityDto<'a, 'b, N, T>| [parent([map(base_int)] 0, [map(another_base_int)] 1)] 0: TupleBase, [map(base_entity_int)] 1, [map(base_string)] 2)]
    TupleBaseEntity<'a, T>,

    #[parent(TupleEntityDto<'a, 'b, N, T>| [map(5)] 0, [map(6)] 1, [map(7)] 2)]
    #[parent(EntityDto<'a, 'b, N, T>| [map(child_int)]0, [map(another_child_int)] 1, [map(child_string)] 2)]
    TupleChild<'b>
);

struct BaseEntity<'a, T : Copy> {
    base: Base<T>,
    base_entity_int: T,
    base_string: &'a str
}

struct TupleBaseEntity<'a, T : Copy>(TupleBase<T>, T, &'a str);

struct Base<T> {
    base_int_2: T,
    another_base_int: i16,
}

struct TupleBase<T>(T, i16);

struct Child<'b> {
    child_int: i32,
    another_child_int: i16,
    child_string: &'b str
}

struct TupleChild<'b>(i32, i16, &'b str);

#[derive(Default)]
struct EntityDto<'a, 'b, const N: usize = 10, T : Copy = i32> {
    parent_int: T,
    base_int: T,
    another_base_int: i16,
    base_entity_int: T,
    base_string: &'a str,
    child_int: i32,
    another_child_int: i16,
    child_string: &'b str
}

#[derive(Default)]
struct TupleEntityDto<'a, 'b, const N: usize = 10, T : Copy = i32>(T, T, i16, T, &'a str, i32, i16, &'b str);

#[test]
fn named2named() {
    let dto = EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        base_string: "hello",
        child_int: 789,
        another_child_int: 987,
        child_string: "child"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!("hello", entity.base.base_string);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
    assert_eq!("child", entity.child.child_string);
}

#[test]
fn named2unnamed() {
    let dto = EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        base_string: "hello",
        child_int: 789,
        another_child_int: 987,
        child_string: "child",
    };

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1 .0 .0);
    assert_eq!(456, entity.1 .0 .1);
    assert_eq!(654, entity.1 .1);
    assert_eq!("hello", entity.1 .2);
    assert_eq!(789, entity.2 .0);
    assert_eq!(987, entity.2 .1);
    assert_eq!("child", entity.2 .2);
}

#[test]
fn named2named_reverse() {
    let entity: Entity<1, i32> = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let dto: EntityDto<1, i32> = entity.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!("hello", dto.base_string);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
    assert_eq!("child", dto.child_string);
}

#[test]
fn named2unnamed_reverse() {
    let entity: Entity<3, i32> = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base { base_int_2: 321, another_base_int: 456 },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let dto: TupleEntityDto<3, i32> = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!("hello", dto.4);
    assert_eq!(789, dto.5);
    assert_eq!(987, dto.6);
    assert_eq!("child", dto.7)
}

#[test]
fn named2named_ref() {
    let dto = &EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        base_string: "hello",
        child_int: 789,
        another_child_int: 987,
        child_string: "child",
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, entity.parent_int);
    assert_eq!(dto.base_int, entity.base.base.base_int_2);
    assert_eq!(dto.another_base_int, entity.base.base.another_base_int);
    assert_eq!(dto.base_entity_int, entity.base.base_entity_int);
    assert_eq!(dto.base_string, entity.base.base_string);
    assert_eq!(dto.child_int, entity.child.child_int);
    assert_eq!(dto.another_child_int, entity.child.another_child_int);
    assert_eq!(dto.child_string, entity.child.child_string);
}

#[test]
fn named2unnamed_ref() {
    let dto = &EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        base_string: "hello",
        child_int: 789,
        another_child_int: 987,
        child_string: "child",
    };

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, entity.0);
    assert_eq!(dto.base_int, entity.1 .0 .0);
    assert_eq!(dto.another_base_int, entity.1 .0 .1);
    assert_eq!(dto.base_entity_int, entity.1 .1);
    assert_eq!(dto.base_string, entity.1 .2);
    assert_eq!(dto.child_int, entity.2 .0);
    assert_eq!(dto.another_child_int, entity.2 .1);
    assert_eq!(dto.child_string, entity.2 .2);
}

#[test]
fn named2named_reverse_ref() {
    let entity: &Entity<12, i32> = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let dto: EntityDto<12, i32> = entity.try_into().unwrap();

    assert_eq!(entity.parent_int, dto.parent_int);
    assert_eq!(entity.base.base.base_int_2, dto.base_int);
    assert_eq!(entity.base.base.another_base_int, dto.another_base_int);
    assert_eq!(entity.base.base_entity_int, dto.base_entity_int);
    assert_eq!(entity.base.base_string, dto.base_string);
    assert_eq!(entity.child.child_int, dto.child_int);
    assert_eq!(entity.child.another_child_int, dto.another_child_int);
    assert_eq!(entity.child.child_string, dto.child_string);
}

#[test]
fn named2unnamed_reverse_ref() {
    let entity: &Entity<7, i32> = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base { base_int_2: 321, another_base_int: 456 },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let dto: TupleEntityDto<7, i32> = entity.try_into().unwrap();

    assert_eq!(entity.parent_int, dto.0);
    assert_eq!(entity.base.base.base_int_2, dto.1);
    assert_eq!(entity.base.base.another_base_int, dto.2);
    assert_eq!(entity.base.base_entity_int, dto.3);
    assert_eq!(entity.base.base_string, dto.4);
    assert_eq!(entity.child.child_int, dto.5);
    assert_eq!(entity.child.another_child_int, dto.6);
    assert_eq!(entity.child.child_string, dto.7);
}

#[test]
fn unnamed2unnamed() {
    let dto = TupleEntityDto(123, 321, 456, 654, "hello", 789, 987, "child");

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1 .0 .0);
    assert_eq!(456, entity.1 .0 .1);
    assert_eq!(654, entity.1 .1);
    assert_eq!("hello", entity.1. 2);
    assert_eq!(789, entity.2 .0);
    assert_eq!(987, entity.2 .1);
    assert_eq!("child", entity.2 .2);
}

#[test]
fn unnamed2named() {
    let dto = TupleEntityDto(123, 321, 456, 654, "hello", 789, 987, "child");

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!("hello", entity.base.base_string);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
    assert_eq!("child", entity.child.child_string);
}

#[test]
fn unnamed2unnamed_reverse() {
    let entity: TupleEntity<8, u32>  = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let dto: TupleEntityDto<8, u32> = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!("hello", dto.4);
    assert_eq!(789, dto.5);
    assert_eq!(987, dto.6);
    assert_eq!("child", dto.7);
}

#[test]
fn unnamed2named_reverse() {
    let entity: TupleEntity<9, i16>  = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let dto: EntityDto<9, i16> = entity.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!("hello", dto.base_string);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
    assert_eq!("child", dto.child_string);
}

#[test]
fn unnamed2unnamed_ref() {
    let dto: &TupleEntityDto<11, i32> = &TupleEntityDto(123, 321, 456, 654, "hello", 789, 987, "child");

    let entity: TupleEntity<11, i32> = dto.try_into().unwrap();

    assert_eq!(dto.0, entity.0);
    assert_eq!(dto.1, entity.1 .0 .0);
    assert_eq!(dto.2, entity.1 .0 .1);
    assert_eq!(dto.3, entity.1 .1);
    assert_eq!(dto.4, entity.1. 2);
    assert_eq!(dto.5, entity.2 .0);
    assert_eq!(dto.6, entity.2 .1);
    assert_eq!(dto.7, entity.2 .2);
}

#[test]
fn unnamed2named_ref() {
    let dto: &TupleEntityDto<15, i32> = &TupleEntityDto(123, 321, 456, 654, "hello", 789, 987, "child");

    let entity: Entity<15, i32> = dto.try_into().unwrap();

    assert_eq!(dto.0, entity.parent_int);
    assert_eq!(dto.1, entity.base.base.base_int_2);
    assert_eq!(dto.2, entity.base.base.another_base_int);
    assert_eq!(dto.3, entity.base.base_entity_int);
    assert_eq!(dto.4, entity.base.base_string);
    assert_eq!(dto.5, entity.child.child_int);
    assert_eq!(dto.6, entity.child.another_child_int);
    assert_eq!(dto.7, entity.child.child_string);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let entity: &TupleEntity<20, i32> = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let dto: TupleEntityDto<20, i32> = entity.try_into().unwrap();

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1 .0 .0, dto.1);
    assert_eq!(entity.1 .0 .1, dto.2);
    assert_eq!(entity.1 .1, dto.3);
    assert_eq!(entity.1 .2, dto.4);
    assert_eq!(entity.2 .0, dto.5);
    assert_eq!(entity.2 .1, dto.6);
    assert_eq!(entity.2 .2, dto.7);
}

#[test]
fn unnamed2named_reverse_ref() {
    let entity: &TupleEntity<21, i32> = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let dto: EntityDto<21, i32> = entity.try_into().unwrap();

    assert_eq!(entity.0, dto.parent_int);
    assert_eq!(entity.1 .0 .0, dto.base_int);
    assert_eq!(entity.1 .0 .1, dto.another_base_int);
    assert_eq!(entity.1 .1, dto.base_entity_int);
    assert_eq!(entity.1 .2, dto.base_string);
    assert_eq!(entity.2 .0, dto.child_int);
    assert_eq!(entity.2 .1, dto.another_child_int);
    assert_eq!(entity.2 .2, dto.child_string);
}

#[test]
fn existing_named2named() {
    let entity: Entity<10, usize> = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let mut dto: EntityDto<10, usize> = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!("hello", dto.base_string);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
    assert_eq!("child", dto.child_string);
}

#[test]
fn existing_named2unnamed() {
    let entity: Entity<_, _> = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let mut dto: TupleEntityDto<10, i32> = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!("hello", dto.4);
    assert_eq!(789, dto.5);
    assert_eq!(987, dto.6);
    assert_eq!("child", dto.7);
}

#[test]
fn existing_named2named_ref() {
    let entity: &Entity<10, i32> = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let mut dto: EntityDto<10, i32> = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!("hello", dto.base_string);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
    assert_eq!("child", dto.child_string);
}

#[test]
fn existing_named2unnamed_ref() {
    let entity: &Entity<10, u16> = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
            base_string: "hello",
        },
        child: Child { child_int: 789, another_child_int: 987, child_string: "child" },
    };

    let mut dto: TupleEntityDto<10, u16> = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!("hello", dto.4);
    assert_eq!(789, dto.5);
    assert_eq!(987, dto.6);
    assert_eq!("child", dto.7);
}

#[test]
fn existing_unnamed2unnamed() {
    let entity: TupleEntity<10, i32> = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let mut dto: TupleEntityDto<10, i32> = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!("hello", dto.4);
    assert_eq!(789, dto.5);
    assert_eq!(987, dto.6);
    assert_eq!("child", dto.7);
}

#[test]
fn existing_unnamed2named() {
    let entity: TupleEntity<_, _> = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let entity: &TupleEntity<_, _> = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let mut dto: TupleEntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!("hello", dto.4);
    assert_eq!(789, dto.5);
    assert_eq!(987, dto.6);
    assert_eq!("child", dto.7);
}

#[test]
fn existing_unnamed2named_ref() {
    let entity: &TupleEntity<_, _> = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654, "hello"), TupleChild(789, 987, "child"));

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!("hello", dto.base_string);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
    assert_eq!("child", dto.child_string);
}