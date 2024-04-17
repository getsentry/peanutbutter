# Peanutbutter

A self contained service for keeping track of per-project budgets.

Where `budget` can mean anything depending on the `config`.
It primarily tracks processing time for the `symbolication-native` and `symbolication-js` configs.

## HTTP / JSON Api

- `POST /record_spending`:
  Expects a `{"config_name": "...", "project_id": 1234, "spent": 12.34}` JSON objects as body.
  Records the given `spent` budget for this project.
  Returns a `{"exceeds_budget": false}` JSON response.

- `POST /exceeds_budget`:
  Expects a `{"config_name": "...", "project_id": 1234}` JSON objects as body.

  Returns a `{"exceeds_budget": false}` JSON response.

## Detailed explanation

`Peanutbutter` manages "projects" identified by integer IDs. A project could in principle represent
anything, but the intended use case is Sentry projects. Each project is assigned
a "budget" for a certain time window. Again, the budget could represent any kind of resource,
but the intended use case is processing time.

There are 4 important configuration parameters (which are currently hard-coded):

- `budget`: How much a project is allowed to spend in a fixed time window. Currently 5.0.
- `budgeting_window`: The time window to which the budget applies. Currently 2 minutes.
- `bucket_size`: The size of the time buckets spending gets sorted into. Currently 10 seconds.
- `backoff_duration`: When a project's state changes (from within its budget to exceeding its budget, or the reverse)
  it can't change again for this length of time. Currently 5 minutes.

Taking the default values as an example, each project has a budget of 5.0 units over 2 minutes, with spending
recorded in 10-second blocks. As soon as a project has spent more than 5.0 units total
in the last 2 minutes, it's marked as exceeding its budget and will stay that way for at
least 5 minutes, even if doesn't spend any more. Conversely, once it returns to being within its
budget, it can't be marked as exceeding it again for 5 minutes.
