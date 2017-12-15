#[macro_use]
extern crate exonum;
extern crate exonum_time;
#[macro_use]
extern crate exonum_testkit;
extern crate serde_json;
extern crate serde;

use std::time::{UNIX_EPOCH, SystemTime, Duration};
use exonum::blockchain::{Service, Transaction};
use exonum::crypto::{gen_keypair, PublicKey};
use exonum::encoding;
use exonum::helpers::Height;
use exonum::messages::{Message, RawTransaction, FromRaw};
use exonum::storage::{Fork, MapIndex, Snapshot};
use exonum_time::{TimeService, TimeSchema, TimeProvider, Time};
use exonum_testkit::TestKitBuilder;

const SERVICE_ID: u16 = 128;
const SERVICE_NAME: &str = "marker";
const TX_TIMESTAMP_ID: u16 = 0;

#[derive(Debug)]
pub struct MarkerSchema<T> {
    view: T,
}

impl<T: AsRef<Snapshot>> MarkerSchema<T> {
    pub fn new(view: T) -> Self {
        MarkerSchema { view }
    }

    pub fn marks(&self) -> MapIndex<&Snapshot, PublicKey, i32> {
        MapIndex::new(format!("{}.marks", SERVICE_NAME), self.view.as_ref())
    }
}


impl<'a> MarkerSchema<&'a mut Fork> {
    pub fn messages_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, i32> {
        MapIndex::new(format!("{}.marks", SERVICE_NAME), self.view)
    }
}

message! {
    struct TxMarker {
        const TYPE = SERVICE_ID;
        const ID = TX_TIMESTAMP_ID;
        const SIZE = 48;

        field from:     &PublicKey      [00 => 32]
        field mark:     i32             [32 => 36]
        field time:     SystemTime      [36 => 48]
    }
}

impl Transaction for TxMarker {
    fn verify(&self) -> bool {
        self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        {
            let time_schema = TimeSchema::new(&view);
            // The time in the transactions should be less than in the blockchain.
            match time_schema.time().get() {
                Some(ref current_time) if current_time.time() < self.time() => {
                    return;
                }
                _ => {}
            }
        }
        // Mark the node that generated the transaction.
        let mut schema = MarkerSchema::new(view);
        schema.messages_mut().put(self.from(), self.mark());
    }

    fn info(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

struct MarkerService;

impl Service for MarkerService {
    fn service_name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        match raw.message_type() {
            TX_TIMESTAMP_ID => Ok(Box::new(TxMarker::from_raw(raw)?)),
            _ => {
                let error =
                    encoding::Error::IncorrectMessageType { message_type: raw.message_type() };
                Err(error)
            }
        }
    }
}

#[derive(Debug)]
struct MyTimeProvider;

impl TimeProvider for MyTimeProvider {
    fn current_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::new(10, 0)
    }
}

fn main() {
    // Create testkit for network with one validator
    let mut testkit = TestKitBuilder::validator()
        .with_service(MarkerService)
        .with_service(TimeService::with_provider(
            Box::new(MyTimeProvider) as Box<TimeProvider>,
        ))
        .create();
    // Create two blocks to set the time in the blockchain.
    testkit.create_blocks_until(Height(2));
    // Time, that is saved in storage, will have the value `UNIX_EPOCH + Duration::new(10, 0)`
    let snapshot = testkit.snapshot();
    let time_schema = TimeSchema::new(&snapshot);
    assert_eq!(
        time_schema.time().get(),
        Some(Time::new(UNIX_EPOCH + Duration::new(10, 0)))
    );
    // Create few transactions.
    let keypair1 = gen_keypair();
    let keypair2 = gen_keypair();
    let keypair3 = gen_keypair();
    let tx1 = TxMarker::new(&keypair1.0, 1, UNIX_EPOCH, &keypair1.1);
    let tx2 = TxMarker::new(
        &keypair2.0,
        2,
        UNIX_EPOCH + Duration::new(20, 0),
        &keypair2.1,
    );
    let tx3 = TxMarker::new(
        &keypair3.0,
        3,
        UNIX_EPOCH + Duration::new(5, 0),
        &keypair3.1,
    );
    testkit.create_block_with_transactions(txvec![tx1.clone(), tx2.clone(), tx3.clone()]);
    // Check results.
    let snapshot = testkit.snapshot();
    let schema = MarkerSchema::new(snapshot);
    assert_eq!(schema.marks().get(&keypair1.0), Some(1));
    assert_eq!(schema.marks().get(&keypair2.0), None);
    assert_eq!(schema.marks().get(&keypair3.0), Some(3));
}
