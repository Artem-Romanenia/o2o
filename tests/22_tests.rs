use o2o::o2o;

struct Employee {
    id: i32,
    first_name: String,
    last_name: String,
    subordinate_of: Box<Employee>,
    subordinates: Vec<Box<Employee>>
}
impl Employee {
    fn get_full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(o2o)]
#[map(Employee)]
#[ghost(
    // o2o supports closures with one input parameter.
    // This parameter represents instance on the other side of the conversion.
    first_name: |x| {x.get_first_name()},
    last_name: { @.get_last_name()}
)]
struct EmployeeDto {
    #[map(id)]
    employee_id: i32,
    // '@.' is another flavor of 'inline expression'. 
    // @ also represents instance on the other side of the conversion.
    #[ghost(@.get_full_name())]
    full_name: String,

    #[from(|x| Box::new(x.subordinate_of.as_ref().into()))]
    #[into(subordinate_of, |x| Box::new(x.reports_to.as_ref().into()))]
    reports_to: Box<EmployeeDto>,

    #[map(~.iter().map(|p|Box::new(p.as_ref().into())).collect())]
    subordinates: Vec<Box<EmployeeDto>>
}
impl EmployeeDto {
    fn get_first_name(&self) -> String {
        self.full_name.split_whitespace().collect::<Vec<&str>>()[0].into()
    }
    fn get_last_name(&self) -> String {
        self.full_name.split_whitespace().collect::<Vec<&str>>()[1].into()
    }
}