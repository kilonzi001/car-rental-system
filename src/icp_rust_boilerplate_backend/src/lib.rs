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

// Implement serialization and deserialization for Car and RentalRequest
impl<T> Storable for T
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement bounds for Car and RentalRequest serialization
impl<T> BoundedStorable for T
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
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
    let id = increment_id_counter()?;

    let car = Car {
        id,
        make,
        model,
        year,
        available: true,
    };

    insert_item(&car, &CAR_STORAGE)?;
    Ok(car)
}

#[ic_cdk::update]
fn delete_car(id: u64) -> Result<(), Error> {
    remove_item(&id, &CAR_STORAGE)
}

// Implement query operations for the car rental system
#[ic_cdk::query]
fn get_car(id: u64) -> Result<Car, Error> {
    get_item(&id, &CAR_STORAGE)
}

#[ic_cdk::query]
fn list_cars() -> Vec<Car> {
    list_items(&CAR_STORAGE)
}

#[ic_cdk::update]
fn add_rental_request(
    car_id: u64,
    customer_id: u64,
    start_date: u64,
    end_date: u64,
    status: RentalStatus,
) -> Result<RentalRequest, Error> {
    let id = increment_id_counter()?;

    let rental_request = RentalRequest {
        id,
        car_id,
        customer_id,
        start_date,
        end_date,
        status,
    };

    insert_item(&rental_request, &RENTAL_REQUEST_STORAGE)?;
    Ok(rental_request)
}

#[ic_cdk::update]
fn delete_rental_request(id: u64) -> Result<(), Error> {
    remove_item(&id, &RENTAL_REQUEST_STORAGE)
}

#[ic_cdk::query]
fn get_rental_request(id: u64) -> Result<RentalRequest, Error> {
    get_item(&id, &RENTAL_REQUEST_STORAGE)
}

#[ic_cdk::query]
fn list_rental_requests() -> Vec<RentalRequest> {
    list_items(&RENTAL_REQUEST_STORAGE)
}

#[ic_cdk::query]
fn list_rental_requests_for_car(car_id: u64) -> Vec<RentalRequest> {
    list_items_by_car(car_id)
}

#[ic_cdk::query]
fn list_rental_requests_for_customer(customer_id: u64) -> Vec<RentalRequest> {
    list_items_by_customer(customer_id)
}

#[ic_cdk::update]
fn update_car(id: u64, make: String, model: String, year: u32) -> Result<Car, Error> {
    update_item(
        &id,
        &|car: &mut Car| {
            car.make = make.clone();
            car.model = model.clone();
            car.year = year;
        },
        &CAR_STORAGE,
    )
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
    update_item(
        &id,
        &|rental_request: &mut RentalRequest| {
            rental_request.car_id = car_id;
            rental_request.customer_id = customer_id;
            rental_request.start_date = start_date;
            rental_request.end_date = end_date;
            rental_request.status = status;
        },
        &RENTAL_REQUEST_STORAGE,
    )
}

// Helper function to increment ID counter
fn increment_id_counter() -> Result<u64, Error> {
    ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .map_err(|_| Error::InvalidInput {
            msg: "Failed to increment ID counter".to_string(),
        })
}

// Helper function to insert item into storage
fn insert_item<T>(item: &T, storage: &RefCell<StableBTreeMap<u64, T, Memory>>) -> Result<(), Error>
where
    T: Clone + BoundedStorable,
{
    storage
        .with(|storage| storage.borrow_mut().insert(item.id, item.clone()))
        .map(|_| ())
        .map_err(|_| Error::InvalidInput {
            msg: "Failed to insert item into storage".to_string(),
        })
}

// Helper function to remove item from storage
fn remove_item<T>(id: &u64, storage: &RefCell<StableBTreeMap<u64, T, Memory>>) -> Result<(), Error> {
    storage
        .with(|storage| {
            storage
                .borrow_mut()
                .remove(id)
                .ok_or(Error::NotFound { msg: format!("Item with id={} not found", id) })
        })
        .map(|_| ())
}

// Helper function to get item from storage
fn get_item<T>(id: &u64, storage: &RefCell<StableBTreeMap<u64, T, Memory>>) -> Result<T, Error>
where
    T: Clone,
{
    storage
        .with(|storage| {
            storage
                .borrow()
                .get(id)
                .cloned()
                .ok_or(Error::NotFound { msg: format!("Item with id={} not found", id) })
        })
}

// Helper function to list all items from storage
fn list_items<T>(storage: &RefCell<StableBTreeMap<u64, T, Memory>>) -> Vec<T>
where
    T: Clone,
{
    storage.with(|storage| storage.borrow().values().cloned().collect())
}

// Helper function to list items by car ID
fn list_items_by_car(car_id: u64) -> Vec<RentalRequest> {
    RENTAL_REQUEST_STORAGE
        .with(|storage| {
            storage
                .borrow()
                .values()
                .filter(|request| request.car_id == car_id)
                .cloned()
                .collect()
        })
}

// Helper function to list items by customer ID
fn list_items_by_customer(customer_id: u64) -> Vec<RentalRequest> {
    RENTAL_REQUEST_STORAGE
        .with(|storage| {
            storage
                .borrow()
                .values()
                .filter(|request| request.customer_id == customer_id)
                .cloned()
                .collect()
        })
}

// Helper function to update item in storage
fn update_item<T, F>(
    id: &u64,
    updater: &F,
    storage: &RefCell<StableBTreeMap<u64, T, Memory>>,
) -> Result<T, Error>
where
    T: Clone + BoundedStorable,
    F: Fn(&mut T),
{
    storage
        .with(|storage| {
            storage.borrow_mut().entry(*id).and_modify(|entry| updater(entry));
            storage
                .borrow()
                .get(id)
                .cloned()
                .ok_or(Error::NotFound { msg: format!("Item with id={} not found", id) })
        })
}
// Export the Candid interface
ic_cdk::export_candid!();
