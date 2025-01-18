#[derive(o2o::o2o)]
#[from_owned(String| match @.as_str(), _ => todo!())]
pub enum Gender {
    #[literal("M")]
    Male,
    #[literal("F")]
    Female,
}