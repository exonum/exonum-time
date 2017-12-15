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
