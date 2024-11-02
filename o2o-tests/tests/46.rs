use o2o::o2o;

struct EntityNew {
    same_name: i32,
    another_name: i32,
    action: String,
    action_another_name: i32
}

#[derive(o2o)]
#[owned_into(EntityNewBO)]
#[owned_into_existing(EntityNewBO)]
struct EntityNewWrapper {
	#[parent(
        same_name,
        another_name: different_name,
        action: { ~.to_string() },
        action_another_name: (action_and_diff_name, ~.parse::<i32>().unwrap())
    )]
    wrapped: EntityNew,
    id: i32,
    user_id: i32,
    computed_field: i8,
}

struct EntityNewBO {
    id: i32,
    user_id: i32,
    computed_field: i8,
    same_name: i32,
    different_name: i32,
    action: i32,
    action_and_diff_name: String
}

// impl Into<EntityNewBO> for EntityNewWrapper {
// 	fn into(self) -> EntityNewBO {
// 		EntityNewBO {
// 			id: self.id,
// 			user_id: self.user_id,
// 			computed_field: self.computed_field,
// 			name: self.wrapped.name,
// 			description: self.wrapped.description
// 		}
// 	}
// }

// impl From<EntityNewBO> for EntityNewWrapper {
// 	fn from(value: EntityNewBO) -> Self {
// 		EntityNewWrapper {
// 			id: value.id,
// 			user_id: value.user_id,
// 			computed_field: value.computed_field,
// 			wrapped: EntityNew {
// 				name: value.name,
// 				description: value.description
// 			}
// 		}
// 	}
// }