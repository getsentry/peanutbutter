@0x894cf35c57496706;

interface ProjectBudgets {
    exceedsBudget @0 (request: ExceedsBudgetRequest) -> (reply: ExceedsBudgetReply);
    recordSpending @1 (request: RecordSpendingRequest) -> (reply: ExceedsBudgetReply);
}

struct ExceedsBudgetRequest {
    configName @0 :Text;
    projectId @1 :UInt64;
}

struct RecordSpendingRequest {
    configName @0 :Text;
    projectId @1 :UInt64;
    spent @2 :Float64;
}

struct ExceedsBudgetReply {
    exceedsBudget @0 :Bool;
}
