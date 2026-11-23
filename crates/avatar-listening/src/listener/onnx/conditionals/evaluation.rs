use std::collections::HashSet;

use tract_onnx::pb::{GraphProto, ModelProto, NodeProto, ValueInfoProto};
use tract_onnx::prelude::{
    Framework, InferenceFact, InferenceModelExt, IntoTensor, TValue, TVec, TractResult, onnx,
};
use tract_onnx::tract_core::internal::bail;

pub(super) fn condition_holds(
    proto: &ModelProto,
    position: usize,
    probe: &[TValue],
) -> TractResult<bool> {
    let mut isolated = proto.clone();
    isolate_condition(&mut isolated, position);
    let outputs = run_probe(&isolated, probe)?;
    let Some(flag) = outputs[0].as_slice::<bool>()?.first() else {
        bail!("empty condition");
    };
    Ok(*flag)
}

fn run_probe(proto: &ModelProto, probe: &[TValue]) -> TractResult<TVec<TValue>> {
    let mut model = onnx().model_for_proto_model(proto)?;
    for (index, value) in probe.iter().enumerate() {
        model.set_input_fact(index, InferenceFact::from(value.clone().into_tensor()))?;
    }
    let runnable = model.into_typed()?.into_optimized()?.into_runnable()?;
    runnable.run(probe.iter().cloned().collect())
}

fn isolate_condition(proto: &mut ModelProto, position: usize) {
    let Some(graph) = proto.graph.as_mut() else {
        return;
    };
    let condition = graph.node[position].input[0].clone();
    graph.node = ancestors_of(graph, position, &condition);
    graph.output = vec![ValueInfoProto {
        name: condition,
        ..ValueInfoProto::default()
    }];
}

fn ancestors_of(graph: &GraphProto, position: usize, condition: &str) -> Vec<NodeProto> {
    let mut needed = HashSet::from([condition.to_owned()]);
    let earlier = graph.node[..position].iter().rev();
    let mut kept: Vec<NodeProto> = earlier
        .filter(|node| feeds(node, &mut needed))
        .cloned()
        .collect();
    kept.reverse();
    kept
}

fn feeds(node: &NodeProto, needed: &mut HashSet<String>) -> bool {
    if !node.output.iter().any(|name| needed.contains(name)) {
        return false;
    }
    needed.extend(node.input.iter().cloned());
    true
}
