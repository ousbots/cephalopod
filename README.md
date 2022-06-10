A toy transactions processing program for a take-home test.

## Design

There are two asynchronous functions `parse::input` and `engine::process` that communicate via a
channel as a processing pipeline. The `parse::input` function reads the input csv with a buffered
reader, parses each line, then sends the transaction over the channel. The `engine::process`
function is listening on the channel for transactions, then uses a hashmap as a key-value store
of client accounts and updates the account using the transaction data and a few rules.

This design is meant to simulate a micro-services deployment of a production design with the two
functions representing individual services. The `parse::input` worker emulates a service that reads
csv files and streams them to the processing service, using the channel as a simulated network. Also,
the `engine::process` worker uses a hashmap to simulate how a production service would use a
networked key-value store. Because the bottleneck was reading the csv file, it didn't necessary to
run multiple `engine::process` workers. If that was needed, the hashmap could be moved out of the
function and shared among the workers with an `Arc` and a mutex, but that would then lose the
guarantee of processing transactions chronologically without moving management of the hashmap to it's
own process and adding more functionality that seemed out of the scope of this project. Additionally
in a production environment, the `engine::process` worker would be broken down further to
asynchronously to have threads that handle connections and receiving data and passing that via 
channels to threads that process the transactions and update the key-value store.


## Notes

I tried to refer to the Uniform Commercial Code as much as possible to decide what rules to apply
when processing a transaction, summarized below:

* A deposit is allowed no matter what, even if the account is locked.
* A withdrawal is only allowed on an account that is not locked and has a positive balance.
* Any dispute transaction (dispute, resolve, chargeback) will happen as long as the account exists.
  Even if the account is locked or does not have sufficient funds. This is from my interpretation
  (corroborated via random websites) of UCC section 4-214 (https://www.law.cornell.edu/ucc/4/4-214)
  that the credit dispute process is completely unavoidable no matter what the account status is.


## Testing

The `data/create_data.sh` script is provided to create two large test files, one that makes
deposits to a single account and another to make deposits to many accounts. This is useful
for testing performance and memory usage. There is also the `data/run_test.sh` script to run
a simple end-to-end test to test input parsing and output formatting. Integration tests in the
`tests` folder are used to ensure the correctness of the transaction processing based on the
transaction notes listed above.