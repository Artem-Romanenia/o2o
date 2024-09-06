use o2o::traits::TryIntoExisting;

#[test]
fn from_ref() {
    #[derive(Default)]
    pub struct Entity {
        some_int: i16,
        pub some_str: String,
        pub another_str: String,
    }
    
    #[derive(o2o::o2o)]
    #[owned_try_into(Entity, String)]
    #[try_map_ref(Entity, String)]
    #[try_into_existing(Entity, String)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        #[into(~.to_string())]
        #[from(~.as_str())]
        pub some_str: &'a str,
        #[into(another_str, ~.to_string())]
        #[from(another_str, ~.as_str())]
        #[owned_into_existing(another_str, "123".into())]
        #[ref_into_existing(another_str, "321".into())]
        pub different_str: &'b str,
    }

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("123", entity.another_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("321", entity.another_str);
}

#[test]
fn ref_into() {
    #[derive(o2o::o2o)]
    #[try_from_owned(EntityDto<'a, 'b>, String)]
    #[try_map_ref(EntityDto<'a, 'b>, String)]
    #[ref_try_into_existing(EntityDto<'a, 'b>, String)]
    pub struct Entity {
        some_int: i16,
        #[into(~.as_str())]
        #[from(~.to_string())]
        pub some_str: String,
        #[into(different_str, ~.as_str())]
        #[from(different_str, ~.to_string())]
        #[ref_into_existing(different_str, "123".into())]
        pub another_str: String,
    }

    #[derive(Default)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub different_str: &'b str,
    }

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let entity = Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("123", dto.different_str);

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("123", dto.different_str);
}

#[test]
fn lt2lt() {
    #[derive(Default)]
    pub struct Entity<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub another_str: &'b str,
    }

    #[derive(o2o::o2o)]
    #[try_map(Entity<'a, 'b>, String)]
    #[try_into_existing(Entity<'a, 'b>, String)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        #[map(another_str)]
        #[owned_into_existing(another_str, "123".into())]
        #[ref_into_existing(another_str, "321".into())]
        pub different_str: &'b str,
    }

    let entity = Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("123", entity.another_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("321", entity.another_str);
}

#[test]
fn lt2lt_2() {
    #[derive(Default)]
    pub struct Entity<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub another_str: &'b str,
    }

    #[derive(o2o::o2o)]
    #[try_map(Entity<'c, 'd>, String)]
    #[try_into_existing(Entity<'c, 'd>, String)]
    #[where_clause('c: 'a, 'd: 'b, 'a: 'c, 'b: 'd)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        #[map(another_str)]
        #[owned_into_existing(another_str, "123".into())]
        #[ref_into_existing(another_str, "321".into())]
        pub different_str: &'b str,
    }

    let entity = Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("123", entity.another_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("321", entity.another_str);
}

#[test]
fn lt2lt_3() {
    #[derive(Default)]
    pub struct Entity<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub another_str: &'b str,
    }

    #[derive(Default)]
    pub struct EntityModel<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        pub another_str: &'b str,
    }

    #[derive(o2o::o2o)]
    #[try_map(Entity<'c, 'd>, String)]
    #[try_into_existing(Entity<'c, 'd>, String)]
    #[try_map(EntityModel<'cc, 'dd>, String)]
    #[try_into_existing(EntityModel<'cc, 'dd>, String)]
    #[where_clause(Entity<'c, 'd>| 'c: 'a, 'd: 'b, 'a: 'c, 'b: 'd)]
    #[where_clause(EntityModel<'cc, 'dd>| 'cc: 'a, 'dd: 'b, 'a: 'cc, 'b: 'dd)]
    pub struct EntityDto<'a, 'b> {
        some_int: i16,
        pub some_str: &'a str,
        #[map(another_str)]
        #[owned_into_existing(another_str, "123".into())]
        #[ref_into_existing(another_str, "321".into())]
        pub different_str: &'b str,
    }

    let entity = Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let entity = &Entity {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("B", entity.another_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("123", entity.another_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut entity: Entity = Default::default();
    dto.try_into_existing(&mut entity).unwrap();

    assert_eq!(123, entity.some_int);
    assert_eq!("A", entity.some_str);
    assert_eq!("321", entity.another_str);

    let model = EntityModel {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = model.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let model: EntityModel = dto.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!("A", model.some_str);
    assert_eq!("B", model.another_str);

    let model = &EntityModel {
        some_int: 123,
        some_str: "A".into(),
        another_str: "B".into()
    };

    let dto: EntityDto = model.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!("A", dto.some_str);
    assert_eq!("B", dto.different_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let model: EntityModel = dto.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!("A", model.some_str);
    assert_eq!("B", model.another_str);

    let dto = EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut model: EntityModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!("A", model.some_str);
    assert_eq!("123", model.another_str);

    let dto = &EntityDto {
        some_int: 123,
        some_str: "A",
        different_str: "B"
    };

    let mut model: EntityModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!("A", model.some_str);
    assert_eq!("321", model.another_str);
}