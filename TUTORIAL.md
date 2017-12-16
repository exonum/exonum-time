# Exonum-time tutorial

Exonum-time is a time oracle service for [Exonum blockchain framework](https://exonum.com/).
This service allows to determine time, 
import it from the external world to the blockchain 
and keep its current value in the blockchain.

Below is a simple user guide.

* [How it works](#how-it-works)
* [Usage](#usage)

## How it works

## Usage

The service data schema consists of two indexes:

+ **current_time: Entry&lt;Time>** - the consolidated time, which is stored in the blockchain,
+ **validators_time: MapIndex&lt;PublicKey, SystemTime>** - the last known local time on the nodes of the validators.

Typical usage of the service boils down to importing the schema and calling its `current_time()` or `validators_time()` methods.

Below is an example of a method for processing a transaction, 
which must be executed no later than the specified time 
(this time is written in the transaction body by a separate field):

```rust
message! {
	struct Tx {
		…
		field time: SystemTime	 [00 => 12]
		…
	}
}
 
impl Transaction for Tx {
	…
	fn execute(&self, view: &mut Fork) {
		// Import schema.
		let time_schema = TimeSchema::new(&view);
		// The time in the transaction should be less than in the blockchain.
		match time_schema.current_time().get() {
			Some(ref current_time) if current_time.time() < self.time() => {
				return;
			}
			…
		}
		…
	}
	… 
}
```

Similarly to obtaining the consolidated time, 
you can get the known time for the nodes of validators:

```rust
let time_schema = TimeSchema::new(&view);
let validators_time = time_schema.validators_time().
```

The full implementation of the service, which uses the time oracle, 
is in the [`directory`](https://github.com/exonum/exonum-time/tree/master/examples).
For testing the service [`exonum-testkit`][exonum-testkit] is used.

Also in the service the API for Read Requests is implemented.
The time oracle has one endpoint in the Public API and one in the Private API:
+ `GET /current_time` - returns consolidated time through Public API;
+ `GET /validators_time` - returns the last known time on the nodes of the validators through Private API.

[exonum-testkit]: https://github.com/exonum/exonum-testkit
