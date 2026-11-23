mod agent_gate_replay;
mod output_properties;
mod routing_checks;
mod runtime;
mod speak_replay;
mod trace;

type TestResult = Result<(), Box<dyn std::error::Error>>;
