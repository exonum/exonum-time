# exonum-time

[![Build Status](https://travis-ci.com/exonum/exonum-time.svg?branch=master)](https://travis-ci.com/exonum/exonum-time)

Exonum-time is a time oracle service for [Exonum blockchain framework](https://exonum.com/).
This service allows to determine time, 
import it from the external world to the blockchain 
and keep its current value in the blockchain.

* [Installation](#installation)
* [License](#license)

## Installation

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

## License

`exonum-time` is licensed under the Apache License (Version 2.0). See [LICENSE](https://github.com/exonum/exonum-time/blob/master/LICENSE) for details.
