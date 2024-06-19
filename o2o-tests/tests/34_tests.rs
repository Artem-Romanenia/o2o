use std::num::TryFromIntError;

struct Entity {
    x: i32
}

#[derive(o2o::o2o)]
// #[try_map(Entity, TryFromIntError)]
#[try_into_existing(Entity, TryFromIntError)]
struct EntityDto {
    #[try_map(~.try_into()?)]
    x: i32
}

// impl std::convert::TryInto<Entity> for EntityDto {
//     type Error = TryFromIntError;

//     fn try_into(self) -> Result<Entity, TryFromIntError> {
//         Ok(Entity {
//             x: self.x.try_into()?,
//         })
//     }
// }