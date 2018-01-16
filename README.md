# exonum-time

[![Build Status](https://travis-ci.com/exonum/exonum-time.svg?branch=master)](https://travis-ci.com/exonum/exonum-time)

Exonum-time is a time oracle service for [Exonum blockchain framework](https://exonum.com/).
This service allows to determine time, 
import it from the external world to the blockchain 
and keep its current value in the blockchain.

## Usage

Add the following line to the `Cargo.toml`:

```toml
[dependecies]
exonum-time = "0.1.0"
```

And activate service in the main project file:

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

To use your own time provider, you need to implement it and create your own `ServiceFactory`
in the main project file:

```rust
extern crate exonum;
extern crate exonum_time;
 
use std::time::{UNIX_EPOCH, SystemTime};
use exonum::blockchain::Service;
use exonum::helpers::fabric::NodeBuilder;
use exonum::helpers::fabric::{ServiceFactory, Context};
use exonum_time::{TimeService, TimeProvider};
 
#[derive(Debug)]
struct MyTimeProvider;
 
impl TimeProvider for MyTimeProvider {
    fn current_time(&self) -> SystemTime {
        UNIX_EPOCH
    }
}
 
#[derive(Debug)]
struct MyTimeServiceFactory;
 
impl ServiceFactory for MyTimeServiceFactory {
    fn make_service(&mut self, _: &Context) -> Box<Service> {
        Box::new(TimeService::with_provider(
            Box::new(MyTimeProvider) as Box<TimeProvider>,
        ))
    }
}
 
fn main() {
    exonum::helpers::init_logger().unwrap();
    NodeBuilder::new()
        .with_service(Box::new(MyTimeServiceFactory))
        .run();
}
```

### Importing the data schema

Typical usage of the service boils down to importing the schema and calling its `time()` or `validators_time()` methods.

Below is an example of a method for processing a transaction, 
which must be executed no later than the specified time 
(this time is written in the transaction body by in a separate field):

```rust
message! {
    struct Tx {
        ...
        field time: SystemTime	 [00 => 12]
        ...
    }
}
 
impl Transaction for Tx {
    ...
    fn execute(&self, view: &mut Fork) {
        // Import schema.
        let time_schema = exonum_time::TimeSchema::new(&view);
        // The time in the transaction should be less than in the blockchain.
        match time_schema.time().get() {
            Some(ref current_time) if current_time.time() < self.time() => {
                return;
            }
            _ => { ... }
        }
        ...
    }
    ... 
}
```

You can get the time of each validator node in the same manner the consolidated time of the system is obtained:

```rust
let time_schema = exonum_time::TimeSchema::new(&view);
let validators_time = time_schema.validators_time();
```

The full implementation of the service, which uses the time oracle, 
is in the [`directory`][directory].
For testing the service [`exonum-testkit`][exonum-testkit] is used.

### REST API

The service has one endpoint per Public API and Private API:
* [Get current time](#current-time)
* [Get current validators times](#current-validators-times)
* [Get all validators times](#all-validators-times)

All REST endpoints share the same base path, denoted **{base_path}**, equal to `api/services/exonum_time/v1`.

!!! tip See [Service][service] for a description of types of endpoints in the service.

#### Current time

```None
GET {base_path}/current_time
```

Returns consolidated time.

##### Parameters

None.

##### Response

Example of JSON response:

```None
{
  "nanos_since_epoch": 15555000,
  "secs_since_epoch": 1516106164
}
```

`null` is returned if there is no consolidated time.

#### Current validators times

```None
GET {base_path}/validators_times
```

Returns the latest timestamps indicated by current validator nodes.

##### Parameters

None.

##### Response

Example of JSON response:

```None
[
  {
    "public_key": "83955565ee605f68fe334132b5ae33fe4ae9be2d85fbe0bd9d56734ad4ffdebd",
    "time": {
      "nanos_since_epoch": 626107000,
      "secs_since_epoch": 1516011501
    }
  },
  {
    "public_key": "f6753f4b130ce098b1322a6aac6accf2d5770946c6db273eab092197a5320717",
    "time": {
      "nanos_since_epoch": 581130000,
      "secs_since_epoch": 1514209665
    }
  },
  {
    "public_key": "52baa9d4c4029b925cedf1a1515c874a68e9133102d0823a6de88eb9c6694a59",
    "time": null
  }  
]
```

#### All validators times

```None
GET {base_path}/validators_times/all
```

Returns the latest timestamps indicated by all validator nodes for which time is known.

##### Parameters

None.

##### Response

Example of JSON response:

```None
[
  {
    "public_key": "83955565ee605f68fe334132b5ae33fe4ae9be2d85fbe0bd9d56734ad4ffdebd",
    "time": {
      "nanos_since_epoch": 626107000,
      "secs_since_epoch": 1516011501
    }
  },
  {
    "public_key": "f6753f4b130ce098b1322a6aac6accf2d5770946c6db273eab092197a5320717",
    "time": {
      "nanos_since_epoch": 581130000,
      "secs_since_epoch": 1514209665
    }
  }
]
```

## License

`exonum-time` is licensed under the Apache License (Version 2.0). See [LICENSE](https://github.com/exonum/exonum-time/blob/master/LICENSE) for details.

[directory]: examples/simple_service.rs
[exonum-testkit]: https://github.com/exonum/exonum-testkit
[service]: https://github.com/exonum/exonum-doc/blob/master/src/architecture/services.md

