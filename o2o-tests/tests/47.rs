#[derive(o2o::o2o)]
#[map(CarDto)]
#[into_existing(CarDto)]
struct Car {
    number_of_doors: i8,
    #[parent(number_of_seats, [parent([map_ref(~.clone())] brand, year)] machine: Machine)]
    vehicle: Vehicle
}

struct Vehicle {
    number_of_seats: i16,
    machine: Machine,
}

struct Machine {
    brand: String,
    year: i16
}

struct CarDto {
    number_of_doors: i8,
    number_of_seats: i16,
    brand: String,
    year: i16
}