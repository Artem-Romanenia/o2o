#[test]
fn from_ref() {
    pub struct Entity {
        some_int: i16,
        pub some_str: String,
        pub another_str: String,
    }
    
    #[derive(o2o::o2o)]
    #[from_ref(Entity)]
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

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);
}

#[test]
fn ref_into() {
    #[derive(o2o::o2o)]
    #[ref_into(EntityDto<'a, 'b>)]
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

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);
}

#[test]
fn lt2lt() {
    pub struct Entity<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub another_str: &'b str,
    }

    // #[derive(o2o::o2o)]
    // #[from_ref(Entity<'c, 'd>)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        // #[map(another_str)]
        pub different_str: &'b str,
    }

    impl<'a, 'b, 'c, 'd, 'o2o: 'a + 'b> ::core::convert::From<&'o2o Entity<'c, 'd>> for EntityDto<'a, 'b>
    {
        fn from(value: &'o2o Entity<'c, 'd>) -> EntityDto<'a, 'b> {
            EntityDto {
                some_int: value.some_int,
                some_str: value.some_str,
                different_str: value.another_str,
            }
        }
    }
}

#[test]
fn map_ref() {
    pub struct Entity {
        some_int: i16,
        pub some_str: String,
        pub another_str: String,
    }
    
    #[derive(o2o::o2o)]
    #[map_ref(Entity)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        #[from(~.as_str())]
        #[into(~.to_string())]
        pub some_str: &'a str,
        #[from(another_str, ~.as_str())]
        #[into(another_str, ~.to_string())]
        pub different_str: &'b str,
    }

    // impl<'a, 'b, 'o2o> ::core::convert::From<&'o2o Entity> for EntityDto<'a, 'b>
    // where
    //     'o2o: 'a + 'b,
    // {
    //     fn from(value: &'o2o Entity) -> EntityDto<'a, 'b> {
    //         EntityDto {
    //             some_int: value.some_int,
    //             some_str: value.some_str.as_str(),
    //             different_str: value.another_str.as_str(),
    //         }
    //     }
    // }
    // impl<'a, 'b, 'c> ::core::convert::Into<Entity> for &'c EntityDto<'a, 'b>
    // {
    //     fn into(self) -> Entity {
    //         Entity {
    //             some_int: self.some_int,
    //             some_str: self.some_str.to_string(),
    //             another_str: self.different_str.to_string(),
    //         }
    //     }
    // }

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);
}