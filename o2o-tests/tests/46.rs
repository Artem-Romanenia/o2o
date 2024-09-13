use o2o::o2o;

struct EntityNew {
    name: String,
    description: String
}

#[derive(o2o)]
#[map_owned(EntityNewBO)]
struct EntityNewWrapper {
	#[parent(sameName, another_name: different_name, action: { ~.to_str() }, both: (name, ~.to_str()))]
    wrapped: EntityNew,
    id: i32,
    user_id: i32,
    computed_field: i8
}

struct EntityNewBO {
    id: i32,
    user_id: i32,
    computed_field: i8,
    name: String,
    description: String
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