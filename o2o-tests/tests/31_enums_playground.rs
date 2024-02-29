enum EnumWithData {
    Item1(String),
    Item2(i32, i16),
}

enum EnumWithDataDto {
    Item1(String),
    Item2(i32, i16),
}

impl From<EnumWithData> for EnumWithDataDto {
    fn from(value: EnumWithData) -> EnumWithDataDto {
        match value {
            EnumWithData::Item1(str) => EnumWithDataDto::Item1(str),
            EnumWithData::Item2(i, i2) => EnumWithDataDto::Item2(i, i2),
        }
    }
}

// impl Into<EnumWithData> for EnumWithDataDto {
//     fn into(self) -> EnumWithData {
//         match self {
//             EnumWithDataDto::Item1(str) => EnumWithData::Item1(str),
//             EnumWithDataDto::Item2(i) => EnumWithData::Item2(i),
//         }
//     }
// }

// impl From<&EnumWithData> for EnumWithDataDto {
//     fn from(value: &EnumWithData) -> EnumWithDataDto {
//         match value {
//             EnumWithData::Item1(str) => EnumWithDataDto::Item1(str.clone()),
//             EnumWithData::Item2(i) => EnumWithDataDto::Item2(*i),
//         }
//     }
// }

enum Test {
    Opt1(i32, String),
    Opt2 { val: i32, str: String },
    Opt3,
    Opt4,
    Opt5
}

enum Test2 {
    Opt1(i32, String),
    Opt2 { val: i32, str: String},
    SubOpt(Test123)
}

enum Test123 {
    Opt3,
    SubOpt(Test321)
}

enum Test321 {
    Opt4,
    Opt5
}

impl From<Test> for Test2 {
    fn from(value: Test) -> Self {
        match value {
            Test::Opt1(i, s) => Test2::Opt1(i, s),
            Test::Opt2 { val, str } => Test2::Opt2 { val, str },
            Test::Opt3 => Test2::SubOpt(Test123::Opt3),
            Test::Opt4 => Test2::SubOpt(Test123::SubOpt(Test321::Opt4)),
            Test::Opt5 => Test2::SubOpt(Test123::SubOpt(Test321::Opt5)),
        }
    }
}

impl Into<Test> for Test2 {
    fn into(self) -> Test {
        match self {
            Test2::Opt1(i, s) => Test::Opt1(i, s),
            Test2::Opt2 { val, str } => Test::Opt2 { val, str },
            Test2::SubOpt(sub) => match sub {
                Test123::Opt3 => Test::Opt3,
                Test123::SubOpt(sub) => match sub {
                    Test321::Opt4 => Test::Opt4,
                    Test321::Opt5 => Test::Opt5,
                },
            },
        }
    }
}

#[derive(Default/*, o2o::o2o */)]
struct Entity {
    item_1: String,
    item_2: i32,
    item_3: i32
}

#[derive(Default/*, o2o::o2o */)]
//#[map_owned(Entity| update: { Default::default() }, test: {"123"})]
//#[o2o(update_with(Default::default()))]
struct EntityDto {
    //#[map(~.clone())]
    item_1: String,
    item_2: i32,
    //#[ghost({123})]
    
}

impl std::convert::From<Entity> for EntityDto {
    fn from(value: Entity) -> EntityDto {
        EntityDto {
            item_1: value.item_1.clone(),
            item_2: value.item_2
        }
    }
}
impl std::convert::Into<Entity> for EntityDto {
    fn into(self) -> Entity {
        Entity {
            item_1: self.item_1.clone(),
            item_2: self.item_2,
            ..Default::default()
        }
    }
}