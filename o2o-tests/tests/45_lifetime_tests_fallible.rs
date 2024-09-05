#[test]
fn from_ref() {
    pub struct Entity {
        some_int: i16,
        pub some_str: String,
        pub another_str: String,
    }
    
    #[derive(o2o::o2o)]
    #[try_from_ref(Entity, String)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        #[from(~.as_str())]
        pub some_str: &'a str,
        #[from(another_str, ~.as_str())]
        pub different_str: &'b str,
    }

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);
}

#[test]
fn ref_into() {
    #[derive(o2o::o2o)]
    #[ref_try_into(EntityDto<'a, 'b>, String)]
    pub struct Entity {
        some_int: i16,
        #[into(~.as_str())]
        pub some_str: String,
        #[into(different_str, ~.as_str())]
        pub another_str: String,
    }

    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub different_str: &'b str,
    }

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);
}