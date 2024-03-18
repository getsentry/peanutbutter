# Peanutbutter

A gRPC-based service for keeping track of whether projects stay within their
allotted processing time.

## Compiling
Compiling `peanutbutter` requires `protobuf` to be installed.

## Quick start
1. Start the service with `cargo run`. Currently it always runs on `[::1]:50051`.
1. Record spending for projects, for example with `grpcurl`:
    ```
    > grpcurl -plaintext -import-path ./proto -proto project_budget.proto -d '{"config_name": "symbolication.native", "project_id": 1337, "spent": 50.0}' '[::1]:50051' project_budget.ProjectBudgets/RecordSpending
    { "exceedsBudget": true }
    ```
    You don't need to create projects beforehand—they're automatically created when you record spending for them for the first time.
1. Check whether a project has outspent its budget. Again, with `grpcurl`:
    ```
    > grpcurl -plaintext -import-path ./proto -proto project_budget.proto -d '{"config_name": "symbolication.native", "project_id": 1337}' '[::1]:50051' project_budget.ProjectBudgets/ExceedsBudget 
    { "exceedsBudget": true }
    ```

## RPC methods
 
* `rpc RecordSpending (RecordSpendingRequest) returns (ExceedsBudgetReply)`:
   Takes a request of the form
    ```
    message RecordSpendingRequest {
        string config_name = 1;
        uint64 project_id = 2;
        double spent = 3;
    }
    ```
  and records the amount spent for the given project and config name. It returns a response
  of the form
    ```
    message ExceedsBudgetReply {
        bool exceeds_budget = 1;
    }
    ```
  telling you whether the project has exceeded its budget for the current time window.
* `rpc ExceedsBudget (ExceedsBudgetRequest) returns (ExceedsBudgetReply)`:
  Takes a request of the form
    ```
    message ExceedsBudgetRequest {
        string config_name = 1;
        uint64 project_id = 2;
    }
    ```
  and checks whether the project has exceeded its budget for the current time window. Returns the
  same response as `RecordSpending`.

## Detailed explanation
`Peanutbutter` manages "projects" identified by integer IDs. A project could in principle represent
anything, but the intended use case is Sentry projects. Each project is assigned
a "budget" for a certain time window. Again, the budget could represent any kind of resource,
but the intended use case is processing time.

There are 4 important configuration parameters (which are currently hard-coded):
* `budget`: How much a project is allowed to spend in a fixed time window. Currently 5.0.
* `budgeting_window`: The time window to which the budget applies. Currently 2 minutes.
* `bucket_size`: The size of the time buckets spending gets sorted into. Currently 10 seconds.
* `backoff_duration`: When a project's state changes (from within its budget to exceeding its budget, or the reverse)
  it can't change again for this length of time. Currently 5 minutes.

Taking the default values as an example, each project has a budget of 5.0 units over 2 minutes, with spending
recorded in 10-second blocks. As soon as a project has spent more than 5.0 units total
in the last 2 minutes, it's marked as exceeding its budget and will stay that way for at
least 5 minutes, even if doesn't spend any more. Conversely, once it returns to being within its
budget, it can't be marked as exceeding it again for 5 minutes.
