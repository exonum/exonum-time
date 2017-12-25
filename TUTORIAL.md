# Exonum-time tutorial

Exonum-time is a time oracle service for [Exonum blockchain framework](https://exonum.com/).
This service allows to determine time, 
import it from the external world to the blockchain 
and keep its current value in the blockchain.

* [The Problem](#the-problem)
* [Solution](#solution)
    + [Integration with Consensus](#integration-with-consensus)
    + [Time Oracle Service](#time-oracle-service)
* [Requirements](#requirements)
* [Implementation of the Time Oracle Service](#implementation-of-the-time-oracle-service)
* [Usage](#usage)

## The Problem

Implementing the business logic of practical solutions requires that one should be able to navigate the calendar time. 
Said time should meet the following criteria:

* **Reliability**. 
The time value must be tolerant to the malicious behavior of Byzantine nodes and  external attacks.

* **Determinateness**. 
The time must be the same on all nodes to ensure that transactions are executed in a deterministic manner. 
This means that the time should be written in the Exonum blockchain storage. 
Thus, the "current" time will be changing similarly on all nodes during execution of transactions, 
including during their update.

* **Sufficient accuracy**. 
The specified time should be fairly accurate. 
In practice, an acceptable deviation is a few seconds (up to a minute). 
The time in Bitcoin headers is updated every 10 or more minutes and can, in principle, 
differ by hours from the real time, so using Bitcoin as a time oracle is not expedient.

* **Monotony**. 
A pragmatic requirement, which simplifies use of time when implementing the business logic.

## Solution

Two approaches were considered for obtaining reliable time in Exonum: 
_integration with consensus_ and _a time oracle service_.

### Integration with Consensus

The validator includes its local time in each _Precommit_ message. 
At the next height the leader includes _+2/3 Precommits_ of the previous block into the _Propose_. 
The time median of these _Precommits_ is recorded into the header of the block obtained based on this _Propose_.

#### Advantages of Integration with Consensus:

* The time value is indicated directly in the header of each block making it more accessible.

* Time is forcibly updated in the course of consensus operation: 
it is impossible to sabotage update of time without stopping consensus.

* Blockchain is not clogged by the time oracle transactions.

#### Disadvantages of Integration with Consensus:

* The consensus code becomes more complex. 
(Time is included into the consensus logic while anchoring and configuration are not.)

* Time is updated with each block. In the case of a large delay in block acceptance, 
all the transactions therein will be executed with the same time value.

### Time Oracle Service

Each validator at a specific time sends a transaction indicating its local time 
(usually immediately after the commit of each block). 
Exonum storage contains an index with the most current time indicated separately by each validator. 
Said time median is stored in a separate index, 
considered the actual time and is updated after each transaction from any of the validators.

#### Advantages of the Time Oracle Service:

* The logic for time update is placed in a separate plug-in service (modularity).

* In case of a long delay in block acceptance, 
the time will be updated along with the delayed block execution while executing its transactions. 
Said time will be accurate enough with regards to the time of the transaction entry into the pool.

#### Disadvantages of the Time Oracle Service:

* The time value is indicated as an index in the Exorum storage, hence, 
receipt thereof by the client requires additional cryptographic checks.

* Each Exonum block will contain time oracle transactions (usually one for each validator).

**It was decided that the time oracle should be implemented as a separate service to develop it separately 
from the core and not complicate the consensus code.**

## Requirements

**Both solutions assume that the local time on all validator nodes is reliable.** 
If the local time on the validator machine is incorrect, **such node is considered Byzantine**. 
Therefore, both solutions require use of a reliable time source locally on each validator machine. 
The solutions considered here provide only an agreed time on the basis of a reliable local time of the validator nodes, 
taking into account possible malicious behavior of up to 1/3 Byzantine nodes.

To obtain local, reliable time external solutions like tlsdate, roughtime, gps-clock, etc. can be applied.

## Implementation of the Time Oracle Service

**The data schema** of such a service consists of two indices:

* **`time: Entry<Time>`** - is the consolidated time we target at.

* **`validators_time: MapIndex<PublicKey, SystemTime>`** - the last known local time of the validator nodes.

The service implements only one transaction consisting of the actual validator’s time and signed with its key. 
The logic of such transaction execution is as follows:

1. It is checked that _`PublicKey`_ belongs to the validator.

2. The time specified in the transaction is greater than said validator’s time specified in the storage 
(transactions potentially can be executed in the order reverse to their creation order, 
but the time must change monotonously).

3. The time for this validator is updated in the storage.

4. All values ​​from the _`validators_time`_ index are fetched.

5. All non-validator nodes are filtered off by the public keys 
(since the validators list can change, the index may contain time values ​​for non-valid validators).

6. The number of the remaining values must be greater than _`2f + 1`_ 
(where _`f = n / 3`_ - the maximum number of Byzantine validators).

7. The resulting list is sorted down from the largest value to the lowest one.

8. _`f + 1`_ time in the resulting list is taken.

9. If the time from `8.` is larger than _`time`_, the value in the storage is replaced with the resulting value.

Thus, the consolidated time can be updated after each transaction with the actual time from any validator node, 
taking into account the possibility of change in the validators list, 
ensuring monotony of such time flow and being tolerant to the malicious behavior of the Byzantine nodes.

It is clear that in a system with no more than f Byzantine nodes, any time in the _`[f + 1, 2f + 1]`_ interval is:

* either the time of an honest node

* or the time in the interval between the timestamps of two honest nodes 
(and therefore such a time can be considered reliable)

For practical reasons, we always choose the _`f + 1`_ timestamp, 
since this value is reliable and at the same time the most relevant.

Potentially, the validator nodes can generate and send a transaction to update the time any moment, however, 
in the current implementation the nodes send the transaction after commit of each block.

At the time when a new blockchain is launched, 
the consolidated time is unknown until the transactions from at least f + 1 validator nodes are processed. 
Further in the course of blockchain operation this time will strictly grow monotonously.

## Usage

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
        …
        field time: SystemTime	 [00 => 12]
        …
    }
}
 
impl Transaction for Tx {
    …
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
        …
    }
    … 
}
```

You can get the time of each validator node in the same manner the consolidated time of the system is obtained:

```rust
let time_schema = exonum_time::TimeSchema::new(&view);
let validators_time = time_schema.validators_time();
```

The full implementation of the service, which uses the time oracle, 
is in the [`directory`](examples/simple_service.rs).
For testing the service [`exonum-testkit`][exonum-testkit] is used.

### REST API

The service has one endpoint per Public API and Private API:
* [Get current time](#current-time)
* [Get validators time](#validators-time)

All REST endpoints share the same base path, denoted **{base_path}**, equal to `api/services/exonum_time/v1`.

!!! tip See [Service][service] for a description of types of endpoints in service.

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
  "time": {
    "nanos": 328946000,
    "secs": "1514207155"
  }
}
```

`null` is returned if there is no consolidated time.

#### Validators time

```None
GET {base_path}/validators_time
```

Returns the latest timestamps indicated by all validator nodes.

##### Parameters

None.

##### Response

Example of JSON response:

```None
[
  {
    "52baa9d4c4029b925cedf1a1515c874a68e9133102d0823a6de88eb9c6694a59": {
      "time": {
        "nanos": 895228000,
        "secs": "1514209166"
      }
    }
  },
  {
    "f6753f4b130ce098b1322a6aac6accf2d5770946c6db273eab092197a5320717": {
      "time": {
        "nanos": 581130000,
        "secs": "1514209665"
      }
    }
  }
]
```

`"Validators time database is empty"` is returned if the time for the validators is unknown.

[exonum-testkit]: https://github.com/exonum/exonum-testkit
[service]: https://github.com/exonum/exonum-doc/blob/master/src/architecture/services.md
