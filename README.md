# exonum-time

[![Build Status](https://travis-ci.com/exonum/exonum-time.svg?branch=master)](https://travis-ci.com/exonum/exonum-time)

Exonum-time is a time oracle service for [Exonum blockchain framework](https://exonum.com/).
This service allows to determine time, 
import it from the external world to the blockchain 
and keep its current value in the blockchain.

* [Usage](#usage)
* [License](#license)

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

[Read more...][tutorial]

## License

`exonum-time` is licensed under the Apache License (Version 2.0). See [LICENSE](https://github.com/exonum/exonum-time/blob/master/LICENSE) for details.

[tutorial]: TUTORIAL.md
