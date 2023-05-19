use o2o::o2o;

struct Entity {
    some_int: i32,
    children: Vec<Child>,
}
struct Child {
    child_int: i32,
}

#[derive(o2o)]
#[map_owned(Entity)]
struct EntityDto {
    some_int: i32,
    #[owned_into(children, ~.iter().map(|p|p.into()).collect())]
    #[from_owned(@.children.iter().map(|p|p.into()).collect())]
    children_vec: Vec<ChildDto>
}

#[derive(o2o)]
#[map(Child)]
struct ChildDto {
    child_int: i32,
}