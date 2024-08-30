use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct Car {
    number_of_doors: i8,
    vehicle: Vehicle,
}
#[derive(Default)]
struct Vehicle {
    number_of_seats: i16,
    can_fly: bool,
    needs_driver: bool,
    horsepower: i32,
    top_speed: f32,
    machine: Machine,
}
#[derive(Default)]
struct Machine {
    id: i32,
    brand: String,
    year: i16,
    weight: f32,
    length: f32,
    width: f32,
    height: f32,
}

#[derive(o2o)]
#[map(Car)]
#[into_existing(Car)]
#[children(vehicle: Vehicle, vehicle.machine: Machine)]
#[ghosts(vehicle.machine@id: { 321 })]
struct CarDto {
    #[o2o(repeat)]
    #[child(vehicle)]
    number_of_seats: i16,
    can_fly: bool,
    needs_driver: bool,
    horsepower: i32,
    top_speed: f32,
    #[o2o(stop_repeat)]
    #[o2o(repeat(child))]
    #[child(vehicle.machine)]
    #[map(~.clone())]
    brand: String,
    year: i16,
    weight: f32,
    length: f32,
    width: f32,
    height: f32,
    #[o2o(stop_repeat)]
    #[o2o(repeat(ghost))]
    #[ghost({123})]
    useless_param: i32,
    useless_param_2: i32,
    #[o2o(skip_repeat)]
    number_of_doors: i8,
    useless_param_3: i32,
}

#[test]
fn named2named() {
    let car = Car {
        number_of_doors: 2,
        vehicle: Vehicle {
            number_of_seats: 4,
            can_fly: false,
            horsepower: 30,
            needs_driver: true,
            top_speed: 105.0,
            machine: Machine {
                id: 123,
                brand: "Trabant".into(),
                year: 1960,
                weight: 615.0,
                length: 3360.0,
                width: 1500.0,
                height: 1440.0,
            },
        },
    };

    let car_dto: CarDto = car.into();

    assert_eq!(2, car_dto.number_of_doors);
    assert_eq!(4, car_dto.number_of_seats);
    assert_eq!(false, car_dto.can_fly);
    assert_eq!(true, car_dto.needs_driver);
    assert_eq!(30, car_dto.horsepower);
    assert_eq!(105.0, car_dto.top_speed);
    assert_eq!("Trabant", car_dto.brand);
    assert_eq!(1960, car_dto.year);
    assert_eq!(615.0, car_dto.weight);
    assert_eq!(3360.0, car_dto.length);
    assert_eq!(1500.0, car_dto.width);
    assert_eq!(1440.0, car_dto.height);
    assert_eq!(123, car_dto.useless_param);
    assert_eq!(123, car_dto.useless_param_2);
    assert_eq!(123, car_dto.useless_param_3);
}

#[test]
fn named2named_reverse() {
    let car_dto = CarDto {
        number_of_doors: 2,
        number_of_seats: 4,
        can_fly: false,
        needs_driver: true,
        horsepower: 30,
        top_speed: 105.0,
        brand: "Trabant".into(),
        year: 1960,
        weight: 615.0,
        length: 3360.0,
        width: 1500.0,
        height: 1440.0,
        useless_param: 123,
        useless_param_2: 123,
        useless_param_3: 123,
    };

    let car: Car = car_dto.into();

    assert_eq!(2, car.number_of_doors);
    assert_eq!(4, car.vehicle.number_of_seats);
    assert_eq!(false, car.vehicle.can_fly);
    assert_eq!(true, car.vehicle.needs_driver);
    assert_eq!(30, car.vehicle.horsepower);
    assert_eq!(105.0, car.vehicle.top_speed);
    assert_eq!("Trabant", car.vehicle.machine.brand);
    assert_eq!(1960, car.vehicle.machine.year);
    assert_eq!(615.0, car.vehicle.machine.weight);
    assert_eq!(3360.0, car.vehicle.machine.length);
    assert_eq!(1500.0, car.vehicle.machine.width);
    assert_eq!(1440.0, car.vehicle.machine.height);
}

#[test]
fn named2named_ref() {
    let car = &Car {
        number_of_doors: 2,
        vehicle: Vehicle {
            number_of_seats: 4,
            can_fly: false,
            horsepower: 30,
            needs_driver: true,
            top_speed: 105.0,
            machine: Machine {
                id: 123,
                brand: "Trabant".into(),
                year: 1960,
                weight: 615.0,
                length: 3360.0,
                width: 1500.0,
                height: 1440.0,
            },
        },
    };

    let car_dto: CarDto = car.into();

    assert_eq!(car.number_of_doors, car_dto.number_of_doors);
    assert_eq!(car.vehicle.number_of_seats, car_dto.number_of_seats);
    assert_eq!(car.vehicle.can_fly, car_dto.can_fly);
    assert_eq!(car.vehicle.needs_driver, car_dto.needs_driver);
    assert_eq!(car.vehicle.horsepower, car_dto.horsepower);
    assert_eq!(car.vehicle.top_speed, car_dto.top_speed);
    assert_eq!(car.vehicle.machine.brand, car_dto.brand);
    assert_eq!(car.vehicle.machine.year, car_dto.year);
    assert_eq!(car.vehicle.machine.weight, car_dto.weight);
    assert_eq!(car.vehicle.machine.length, car_dto.length);
    assert_eq!(car.vehicle.machine.width, car_dto.width);
    assert_eq!(car.vehicle.machine.height, car_dto.height);
    assert_eq!(123, car_dto.useless_param);
    assert_eq!(123, car_dto.useless_param_2);
    assert_eq!(123, car_dto.useless_param_3);
}

#[test]
fn named2named_reverse_ref() {
    let car_dto = &CarDto {
        number_of_doors: 2,
        number_of_seats: 4,
        can_fly: false,
        needs_driver: true,
        horsepower: 30,
        top_speed: 105.0,
        brand: "Trabant".into(),
        year: 1960,
        weight: 615.0,
        length: 3360.0,
        width: 1500.0,
        height: 1440.0,
        useless_param: 123,
        useless_param_2: 123,
        useless_param_3: 123,
    };

    let car: Car = car_dto.into();

    assert_eq!(car_dto.number_of_doors, car.number_of_doors);
    assert_eq!(car_dto.number_of_seats, car.vehicle.number_of_seats);
    assert_eq!(car_dto.can_fly, car.vehicle.can_fly);
    assert_eq!(car_dto.needs_driver, car.vehicle.needs_driver);
    assert_eq!(car_dto.horsepower, car.vehicle.horsepower);
    assert_eq!(car_dto.top_speed, car.vehicle.top_speed);
    assert_eq!(car_dto.brand, car.vehicle.machine.brand);
    assert_eq!(car_dto.year, car.vehicle.machine.year);
    assert_eq!(car_dto.weight, car.vehicle.machine.weight);
    assert_eq!(car_dto.length, car.vehicle.machine.length);
    assert_eq!(car_dto.width, car.vehicle.machine.width);
    assert_eq!(car_dto.height, car.vehicle.machine.height);
}

#[test]
fn existing_named2named() {
    let car_dto = CarDto {
        number_of_doors: 2,
        number_of_seats: 4,
        can_fly: false,
        needs_driver: true,
        horsepower: 30,
        top_speed: 105.0,
        brand: "Trabant".into(),
        year: 1960,
        weight: 615.0,
        length: 3360.0,
        width: 1500.0,
        height: 1440.0,
        useless_param: 123,
        useless_param_2: 123,
        useless_param_3: 123,
    };

    let mut car: Car = Default::default();
    car_dto.into_existing(&mut car);

    assert_eq!(2, car.number_of_doors);
    assert_eq!(4, car.vehicle.number_of_seats);
    assert_eq!(false, car.vehicle.can_fly);
    assert_eq!(true, car.vehicle.needs_driver);
    assert_eq!(30, car.vehicle.horsepower);
    assert_eq!(105.0, car.vehicle.top_speed);
    assert_eq!("Trabant", car.vehicle.machine.brand);
    assert_eq!(1960, car.vehicle.machine.year);
    assert_eq!(615.0, car.vehicle.machine.weight);
    assert_eq!(3360.0, car.vehicle.machine.length);
    assert_eq!(1500.0, car.vehicle.machine.width);
    assert_eq!(1440.0, car.vehicle.machine.height);
}

#[test]
fn existing_named2named_reverse() {
    let car_dto = &CarDto {
        number_of_doors: 2,
        number_of_seats: 4,
        can_fly: false,
        needs_driver: true,
        horsepower: 30,
        top_speed: 105.0,
        brand: "Trabant".into(),
        year: 1960,
        weight: 615.0,
        length: 3360.0,
        width: 1500.0,
        height: 1440.0,
        useless_param: 123,
        useless_param_2: 123,
        useless_param_3: 123,
    };

    let mut car: Car = Default::default();
    car_dto.into_existing(&mut car);

    assert_eq!(car_dto.number_of_doors, car.number_of_doors);
    assert_eq!(car_dto.number_of_seats, car.vehicle.number_of_seats);
    assert_eq!(car_dto.can_fly, car.vehicle.can_fly);
    assert_eq!(car_dto.needs_driver, car.vehicle.needs_driver);
    assert_eq!(car_dto.horsepower, car.vehicle.horsepower);
    assert_eq!(car_dto.top_speed, car.vehicle.top_speed);
    assert_eq!(car_dto.brand, car.vehicle.machine.brand);
    assert_eq!(car_dto.year, car.vehicle.machine.year);
    assert_eq!(car_dto.weight, car.vehicle.machine.weight);
    assert_eq!(car_dto.length, car.vehicle.machine.length);
    assert_eq!(car_dto.width, car.vehicle.machine.width);
    assert_eq!(car_dto.height, car.vehicle.machine.height);
}
