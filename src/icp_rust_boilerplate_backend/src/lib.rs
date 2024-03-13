#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Define type aliases for memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define the structure for a car
#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct Car {
    id: u64,
    make: String,
    model: String,
    year: u32,
    available: bool,
}

// Define the structure for a rental request
#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct RentalRequest {
    id: u64,
    car_id: u64,
    customer_id: u64,
    start_date: u64,
    end_date: u64,
    status: RentalStatus, // Pending, Active, Completed, Canceled
}

// Define the possible statuses for a rental request
#[derive(Debug, PartialEq, candid::CandidType, Deserialize, Serialize, Clone)]
enum RentalStatus {
    Pending,
    Active,
    Completed,
    Canceled,
}

// Implement serialization and deserialization for Car
impl Storable for Car {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement bounds for Car serialization
impl BoundedStorable for Car {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Implement serialization and deserialization for RentalRequest
impl Storable for RentalRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement bounds for RentalRequest serialization
impl BoundedStorable for RentalRequest {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for memory management, ID counter, car storage, and rental request storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static CAR_STORAGE: RefCell<StableBTreeMap<u64, Car, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static RENTAL_REQUEST_STORAGE: RefCell<StableBTreeMap<u64, RentalRequest, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

// Define the possible errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
}

// Implement CRUD operations for cars
#[ic_cdk::update]
fn add_car(make: String, model: String, year: u32) -> Result<Car, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let car = Car {
        id,
        make,
        model,
        year,
        available: true,
    };

    CAR_STORAGE.with(|storage| storage.borrow_mut().insert(id, car.clone()));
    Ok(car)
}

#[ic_cdk::update]
fn delete_car(id: u64) -> Result<(), Error> {
    match CAR_STORAGE.with(|storage| storage.borrow_mut().remove(&id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("Car with id={} not found", id),
        }),
    }
}

// Implement query operations for the car rental system
#[ic_cdk::query]
fn get_car(id: u64) -> Result<Car, Error> {
    match CAR_STORAGE.with(|storage| storage.borrow().get(&id)) {
        Some(car) => Ok(car.clone()),
        None => Err(Error::NotFound {
            msg: format!("Car with id={} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_rental_request(id: u64) -> Result<RentalRequest, Error> {
    match RENTAL_REQUEST_STORAGE.with(|storage| storage.borrow().get(&id)) {
        Some(rental_request) => Ok(rental_request.clone()),
        None => Err(Error::NotFound {
            msg: format!("Rental request with id={} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn list_cars() -> Vec<Car> {
    CAR_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, car)| car.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn list_rental_requests() -> Vec<RentalRequest> {
    RENTAL_REQUEST_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, request)| request.clone())
            .collect()
    })
}

#[ic_cdk::update]
fn add_rental_request(
    car_id: u64,
    customer_id: u64,
    start_date: u64,
    end_date: u64,
    status: RentalStatus,
) -> Result<RentalRequest, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let rental_request = RentalRequest {
        id,
        car_id,
        customer_id,
        start_date,
        end_date,
        status,
    };

    RENTAL_REQUEST_STORAGE
        .with(|storage| storage.borrow_mut().insert(id, rental_request.clone()));

    Ok(rental_request)
}

#[ic_cdk::update]
fn delete_rental_request(id: u64) -> Result<(), Error> {
    match RENTAL_REQUEST_STORAGE.with(|storage| storage.borrow_mut().remove(&id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("Rental request with id={} not found", id),
        }),
    }
}


#[ic_cdk::query]
fn list_rental_requests_for_car(car_id: u64) -> Vec<RentalRequest> {
    RENTAL_REQUEST_STORAGE
        .with(|storage| {
            storage
                .borrow()
                .iter()
                .filter_map(|(_, request)| {
                    if request.car_id == car_id {
                        Some(request.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
}

#[ic_cdk::query]
fn list_rental_requests_for_customer(customer_id: u64) -> Vec<RentalRequest> {
    RENTAL_REQUEST_STORAGE
        .with(|storage| {
            storage
                .borrow()
                .iter()
                .filter_map(|(_, request)| {
                    if request.customer_id == customer_id {
                        Some(request.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
}

#[ic_cdk::update]
fn update_car(id: u64, make: String, model: String, year: u32) -> Result<Car, Error> {
    match CAR_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(car) = storage.get(&id) {
            // Create a cloned copy of the car to update
            let mut updated_car = car.clone();
            // Update the car fields
            updated_car.make = make;
            updated_car.model = model;
            updated_car.year = year;
            // Replace the old car with the updated one
            storage.insert(id, updated_car.clone());
            Ok(updated_car)
        } else {
            Err(Error::NotFound {
                msg: format!("Car with id={} not found", id),
            })
        }
    }) {
        Ok(car) => Ok(car),
        Err(e) => Err(e),
    }
}

#[ic_cdk::update]
fn update_rental_request(
    id: u64,
    car_id: u64,
    customer_id: u64,
    start_date: u64,
    end_date: u64,
    status: RentalStatus,
) -> Result<RentalRequest, Error> {
    match RENTAL_REQUEST_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(rental_request) = storage.get(&id) {
            // Create a cloned copy of the rental request to update
            let mut updated_rental_request = rental_request.clone();
            // Update the rental request fields
            updated_rental_request.car_id = car_id;
            updated_rental_request.customer_id = customer_id;
            updated_rental_request.start_date = start_date;
            updated_rental_request.end_date = end_date;
            updated_rental_request.status = status;
            // Replace the old rental request with the updated one
            storage.insert(id, updated_rental_request.clone());
            Ok(updated_rental_request)
        } else {
            Err(Error::NotFound {
                msg: format!("Rental request with id={} not found", id),
            })
        }
    }) {
        Ok(rental_request) => Ok(rental_request),
        Err(e) => Err(e),
    }
}

// Error handling
// Implement error handling for the functions above

// Export the Candid interface
ic_cdk::export_candid!();