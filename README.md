## Car Rental System Documentation

### Overview
The Car Rental System is a decentralized application built on the Internet Computer (IC) platform, designed to manage car rental operations. It provides functionalities for adding, deleting, updating, and querying cars and rental requests. The system aims to streamline the process of renting cars and managing rental requests efficiently.

The application is developed in Rust programming language utilizing the IC Canister SDK, ensuring secure and decentralized management of car rental data. It leverages stable data structures for efficient storage and retrieval of information, providing a reliable platform for car rental businesses.

### Table of Contents
1. [Dependencies](#dependencies)
2. [Data Structures](#data-structures)
3. [Functions](#functions)
4. [Usage](#usage)
5. [Setting Up the Project](#setup)

### Dependencies <a name="dependencies"></a>
- `serde`: Serialization and deserialization library for Rust.
- `candid`: Library for Candid serialization and deserialization.
- `ic_stable_structures`: Library providing stable data structures for the Internet Computer.
- `std`: Standard library for Rust.

### Data Structures <a name="data-structures"></a>
#### Structs
1. `Car`: Represents a car with fields including ID, make, model, year, and availability status.
2. `RentalRequest`: Represents a rental request with fields including ID, car ID, customer ID, start date, end date, and status.

#### Enums
1. `RentalStatus`: Represents the possible statuses for a rental request including Pending, Active, Completed, and Canceled.

### Functions <a name="functions"></a>
The Car Rental System provides various functions for managing cars and rental requests. Some key functions include:
- `add_car`: Add a new car to the system.
- `delete_car`: Delete a car from the system.
- `get_car`: Get details of a specific car.
- `list_cars`: List all cars available in the system.
- `add_rental_request`: Add a new rental request to the system.
- `delete_rental_request`: Delete a rental request from the system.
- `get_rental_request`: Get details of a specific rental request.
- `list_rental_requests`: List all rental requests in the system.
- `list_rental_requests_for_car`: List all rental requests associated with a specific car.
- `list_rental_requests_for_customer`: List all rental requests associated with a specific customer.
- `update_car`: Update details of an existing car.
- `update_rental_request`: Update details of an existing rental request.

### Usage <a name="usage"></a>
The Car Rental System offers a user-friendly interface for car rental businesses to manage their operations. Users can add, delete, update, and query cars and rental requests seamlessly through the provided functions. Proper error handling is implemented to handle cases such as invalid input or missing data.

### Setting Up the Project <a name="setup"></a>
To set up and start working on the Car Rental System project, follow these steps:

1. **Install Rust and Dependencies**
   - Ensure you have Rust installed, version 1.64 or higher. You can install it using the following commands:
     ```bash
     $ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
     $ source "$HOME/.cargo/env"
     ```
   - Install the `wasm32-unknown-unknown` target:
     ```bash
     $ rustup target add wasm32-unknown-unknown
     ```
   - Install `candid-extractor`:
     ```bash
     $ cargo install candid-extractor
     ```

2. **Install DFINITY SDK (`dfx`)**
   - Install `dfx` using the following commands:
     ```bash
     $ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
     $ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
     $ source ~/.bashrc
     $ dfx start --background
     ```

3. **Update Dependencies**
   - Update the `dependencies` block in `/src/{canister_name}/Cargo.toml` with the required dependencies.

4. **Autogenerate DID**
   - Add the provided script to the root directory of the project.
   - Update line 16 with the name of your canister.
   - Run the script each time you modify/add/remove exported functions of the canister.

5. **Running the Project Locally**
   - Start the replica, running in the background:
     ```bash
     $ dfx start --background
     ```
   - Deploy your canisters to the replica and generate your Candid interface:
     ```bash
     $ npm run gen-deploy
     ```