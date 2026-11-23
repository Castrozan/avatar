use serde_json::Value;

#[path = "quiet/closing.rs"]
mod closing;
#[path = "quiet/script.rs"]
mod script;

pub(super) fn handle(name: &str, request: &Value) -> Result<Value, String> {
    match name {
        "run_script" => script::run(request),
        "close_when_quiet" => closing::close_when_quiet(request),
        "close_when_the_call_ended" => closing::close_when_the_call_ended(request),
        _ => Err(format!("unknown quiet operation {name}")),
    }
}
