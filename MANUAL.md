# Exonum-time manual

Exonum-time is the time oracle service for Exonum blockchain framework.
This service allows to find time 
and submit it from the external world to the blockchain 
and keep its current value in the blockchain. 

Below is a simple user guide.

* [How it works](#how-it-works)
* [Installation](#installation)
* [Testing](#testing)

## How it works

## Installation

Add a following line to the `Cargo.toml`:

```toml
[dev-dependecies]
exonum-time = "1.0"
```

And activate service in main project file:

```rust
extern crate exonum;
extern crate exonum_time;

use exonum::helpers::fabric::NodeBuilder;
use exonum_time::TimeServiceFactory;

fn main() {
    exonum::helpers::init_logger().unwrap();
    NodeBuilder::new()
        .with_service(Box::new(TimeServiceFactory))
        .run();
}
```

## Testing

To verify the correct work of the service,
you need to make sure that transactions are created after the commit of block
and the time value, which is stored in the blockchain, is update.
Fot this you can use public and private service API.

For testing the service is used ['exonum-testkit'][exonum-testkit].

```rust
extern crate exonum;
extern crate exonum_time;
extern crate exonum_testkit;
use std::time::{self, SystemTime};
use exonum::helpers::Height;
use exonum_time::{TimeService, TimeSchema, Time, TimeProvider};
use exonum_testkit::TestKitBuilder;
//A struct that provides the node with a current time.
#[derive(Debug)]
struct MyTimeProvider;
impl TimeProvider for MyTimeProvider {
    fn current_time(&self) -> SystemTime {
        time::UNIX_EPOCH
    }
}
#[test]
fn test_exonum_time_service() {
    // Create simple testkit newtwork.
    let mut testkit = TestKitBuilder::validator()
        .with_service(TimeService::with_provider(
            Box::new(MyTimeProvider) as Box<TimeProvider>,
        ))
        .create();
    // Get validator public key.
    let validator_public_key = &testkit.network().validators().to_vec()[0]
        .public_keys()
        .service_key;
    let snapshot = testkit.snapshot();
    let schema = TimeSchema::new(snapshot);
    // Check that blockchain does not contain time.
    assert_eq!(schema.time().get(), None);
    // Check that the time for the validator is unknown.
    assert_eq!(schema.validators_time().get(validator_public_key), None);
    // Create two blocks.
    testkit.create_blocks_until(Height(2));
    let snapshot = testkit.snapshot();
    let schema = TimeSchema::new(snapshot);
    // Check that the time in blockchain and for the validator has been update.
    assert_eq!(schema.time().get(), Some(Time::new(time::UNIX_EPOCH)));
    assert_eq!(
        schema.validators_time().get(validator_public_key),
        Some(Time::new(time::UNIX_EPOCH))
    );
}

```

[exonum-testkit]: https://github.com/exonum/exonum-testkit
