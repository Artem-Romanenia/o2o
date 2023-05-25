use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct Car {
    number_of_doors: i8,
    vehicle: Vehicle
}
#[derive(Default)]
struct Vehicle {
    number_of_seats: i16,
    machine: Machine,
}
#[derive(Default)]
struct Machine {
    id: i32,
    brand: String,
    year: i16,
}

#[derive(o2o)]
#[map(Car)]
#[into_existing(Car)]
#[children(vehicle: Vehicle, vehicle.machine: Machine)]
#[ghost(vehicle.machine@id: || { 321 })]
struct CarDto {
    number_of_doors: i8,

    #[child(vehicle)]
    number_of_seats: i16,

    #[child(vehicle.machine)]
    #[map(~.clone())]
    brand: String,

    #[child(vehicle.machine)]
    year: i16
}

#[derive(Default)]
pub struct EntityBase {
    pub id: u32,
    pub name: String,
}

#[derive(Default)]
pub struct League {
    pub base: EntityBase,
}

#[derive(Default)]
pub struct Division {
    pub base: EntityBase,
    pub league_id: u32,
    pub league: League,
}

#[derive(Default)]
pub struct Team {
    pub base: EntityBase,
    pub division_id: u32,
    pub division: Division,
}

#[derive(o2o)]
#[o2o(
    map(Team),
    into_existing(Team),
    children(base: EntityBase, division: Division, division.base: EntityBase, division.league: League, division.league.base: EntityBase),
    ghost( 
        division_id: |x| { x.division.id },
        division.base@id: |x| { x.division.id },
        division.base@name: |x| { x.division.name.clone() },
        division@league_id: |x| {  x.league.id },
        division.league.base@id: |x| {  x.league.id },
        division.league.base@name: |x| {  x.league.name.clone() }
    ),
)]
pub struct TeamDto {
    #[child(base)]
    id: u32,

    #[child(base)]
    #[map(~.clone())]
	name: String,

    #[ghost(|x| (&x.division.base).into())]
	division: DivisionDto,

    #[ghost(|x| (&x.division.league.base).into())]
	league: LeagueDto,
}

#[derive(o2o)]
#[from(EntityBase)]
pub struct LeagueDto {
    id: u32,
    #[from(~.clone())]
    name: String,
}

#[derive(o2o)]
#[from(EntityBase)]
pub struct DivisionDto {
    id: u32,
    #[from(~.clone())]
    name: String,
}

#[test]
fn named2named() {
    let car = Car  {
        number_of_doors: 2,
        vehicle: Vehicle { 
            number_of_seats: 4, 
            machine: Machine { 
                id: 123, 
                brand: "Trabant".into(), 
                year: 1960
            }
        }
    };

    let car_dto: CarDto = car.into();

    assert_eq!(2, car_dto.number_of_doors);
    assert_eq!(4, car_dto.number_of_seats);
    assert_eq!("Trabant", car_dto.brand);
    assert_eq!(1960, car_dto.year);
}

#[test]
fn named2named_reverse() {
    let car_dto = CarDto  {
        number_of_doors: 2,
        number_of_seats: 4, 
        brand: "Trabant".into(),
        year: 1960
    };

    let car: Car = car_dto.into();

    assert_eq!(2, car.number_of_doors);
    assert_eq!(4, car.vehicle.number_of_seats);
    assert_eq!("Trabant", car.vehicle.machine.brand);
    assert_eq!(1960, car.vehicle.machine.year);
    assert_eq!(321, car.vehicle.machine.id);
}

#[test]
fn named2named_ref() {
    let car = &Car  {
        number_of_doors: 2,
        vehicle: Vehicle { 
            number_of_seats: 4, 
            machine: Machine { 
                id: 123, 
                brand: "Trabant".into(), 
                year: 1960
            }
        }
    };

    let car_dto: CarDto = car.into();

    assert_eq!(car.number_of_doors, car_dto.number_of_doors);
    assert_eq!(car.vehicle.number_of_seats, car_dto.number_of_seats);
    assert_eq!(car.vehicle.machine.brand, car_dto.brand);
    assert_eq!(car.vehicle.machine.year, car_dto.year);
}

#[test]
fn named2named_reverse_ref() {
    let car_dto = &CarDto  {
        number_of_doors: 2,
        number_of_seats: 4, 
        brand: "Trabant".into(),
        year: 1960
    };

    let car: Car = car_dto.into();

    assert_eq!(car_dto.number_of_doors, car.number_of_doors);
    assert_eq!(car_dto.number_of_seats, car.vehicle.number_of_seats);
    assert_eq!(car_dto.brand, car.vehicle.machine.brand);
    assert_eq!(car_dto.year, car.vehicle.machine.year);
    assert_eq!(321, car.vehicle.machine.id);
}

#[test]
fn existing_named2named() {
    let car_dto = CarDto  {
        number_of_doors: 2,
        number_of_seats: 4, 
        brand: "Trabant".into(),
        year: 1960
    };

    let mut car: Car = Default::default();
    car_dto.into_existing(&mut car);

    assert_eq!(2, car.number_of_doors);
    assert_eq!(4, car.vehicle.number_of_seats);
    assert_eq!("Trabant", car.vehicle.machine.brand);
    assert_eq!(1960, car.vehicle.machine.year);
    assert_eq!(321, car.vehicle.machine.id);
}

#[test]
fn existing_named2named_reverse() {
    let car_dto = &CarDto  {
        number_of_doors: 2,
        number_of_seats: 4, 
        brand: "Trabant".into(),
        year: 1960
    };

    let mut car: Car = Default::default();
    car_dto.into_existing(&mut car);

    assert_eq!(car_dto.number_of_doors, car.number_of_doors);
    assert_eq!(car_dto.number_of_seats, car.vehicle.number_of_seats);
    assert_eq!(car_dto.brand, car.vehicle.machine.brand);
    assert_eq!(car_dto.year, car.vehicle.machine.year);
    assert_eq!(321, car.vehicle.machine.id);
}

#[test]
fn named2named_2() {
    let team = Team {
        base: EntityBase {
            id: 123, 
            name: "Test".into()
        },
        division_id: 456,
        division: Division { 
            base: EntityBase { 
                id: 456, 
                name: "TestDivision".into()
            }, 
            league_id: 789, 
            league: League { 
                base: EntityBase { 
                    id: 789, 
                    name: "TestLeague".into()
                }
            }
        }
    };

    let team_dto: TeamDto = team.into();

    assert_eq!(123, team_dto.id);
    assert_eq!("Test", team_dto.name);
    assert_eq!(456, team_dto.division.id);
    assert_eq!("TestDivision", team_dto.division.name);
    assert_eq!(789, team_dto.league.id);
    assert_eq!("TestLeague", team_dto.league.name);
}

#[test]
fn named2named_reverse_2() {
    let team_dto = TeamDto {
        id: 123,
        name: "Test".into(),
        division: DivisionDto {
            id: 456,
            name: "TestDivision".into(),
        },
        league: LeagueDto {
            id: 789,
            name: "TestLeague".into()
        }
    };

    let team: Team = team_dto.into();

    assert_eq!(123, team.base.id);
    assert_eq!("Test", team.base.name);
    assert_eq!(456, team.division_id);
    assert_eq!(456, team.division.base.id);
    assert_eq!("TestDivision", team.division.base.name);
    assert_eq!(789, team.division.league_id);
    assert_eq!(789, team.division.league.base.id);
    assert_eq!("TestLeague", team.division.league.base.name);
}

#[test]
fn named2named_ref_2() {
    let team = &Team {
        base: EntityBase {
            id: 123, 
            name: "Test".into()
        },
        division_id: 456,
        division: Division { 
            base: EntityBase { 
                id: 456, 
                name: "TestDivision".into()
            }, 
            league_id: 789, 
            league: League { 
                base: EntityBase { 
                    id: 789, 
                    name: "TestLeague".into()
                }
            }
        }
    };

    let team_dto: TeamDto = team.into();

    assert_eq!(team.base.id, team_dto.id);
    assert_eq!(team.base.name, team_dto.name);
    assert_eq!(team.division_id, team_dto.division.id);
    assert_eq!(team.division.base.id, team_dto.division.id);
    assert_eq!(team.division.base.name, team_dto.division.name);
    assert_eq!(team.division.league_id, team_dto.league.id);
    assert_eq!(team.division.league.base.id, team_dto.league.id);
    assert_eq!(team.division.league.base.name, team_dto.league.name);
}

#[test]
fn named2named_ref_reverse_2() {
    let team_dto = &TeamDto {
        id: 123,
        name: "Test".into(),
        division: DivisionDto {
            id: 456,
            name: "TestDivision".into(),
        },
        league: LeagueDto {
            id: 789,
            name: "TestLeague".into()
        }
    };

    let team: Team = team_dto.into();

    assert_eq!(team_dto.id, team.base.id);
    assert_eq!(team_dto.name, team.base.name);
    assert_eq!(team_dto.division.id, team.division_id);
    assert_eq!(team_dto.division.id, team.division.base.id);
    assert_eq!(team_dto.division.name, team.division.base.name);
    assert_eq!(team_dto.league.id, team.division.league_id);
    assert_eq!(team_dto.league.id, team.division.league.base.id);
    assert_eq!(team_dto.league.name, team.division.league.base.name);
}

#[test]
fn existing_named2named_2() {
    let team_dto = TeamDto {
        id: 123,
        name: "Test".into(),
        division: DivisionDto {
            id: 456,
            name: "TestDivision".into(),
        },
        league: LeagueDto {
            id: 789,
            name: "TestLeague".into()
        }
    };

    let mut team: Team = Default::default();
    team_dto.into_existing(&mut team);

    assert_eq!(123, team.base.id);
    assert_eq!("Test", team.base.name);
    assert_eq!(456, team.division_id);
    assert_eq!(456, team.division.base.id);
    assert_eq!("TestDivision", team.division.base.name);
    assert_eq!(789, team.division.league_id);
    assert_eq!(789, team.division.league.base.id);
    assert_eq!("TestLeague", team.division.league.base.name);
}

#[test]
fn existing_named2named_ref_2() {
    let team_dto = &TeamDto {
        id: 123,
        name: "Test".into(),
        division: DivisionDto {
            id: 456,
            name: "TestDivision".into(),
        },
        league: LeagueDto {
            id: 789,
            name: "TestLeague".into()
        }
    };

    let mut team: Team = Default::default();
    team_dto.into_existing(&mut team);

    assert_eq!(team_dto.id, team.base.id);
    assert_eq!(team_dto.name, team.base.name);
    assert_eq!(team_dto.division.id, team.division_id);
    assert_eq!(team_dto.division.id, team.division.base.id);
    assert_eq!(team_dto.division.name, team.division.base.name);
    assert_eq!(team_dto.league.id, team.division.league_id);
    assert_eq!(team_dto.league.id, team.division.league.base.id);
    assert_eq!(team_dto.league.name, team.division.league.base.name);
}