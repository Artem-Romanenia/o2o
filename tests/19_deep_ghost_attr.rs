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
#[ghost(vehicle.machine@id: |_| { 321 })]
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