mod decode;
mod tensor;

use std::collections::HashMap;

use tract_onnx::pb::{GraphProto, ModelProto, NodeProto, TensorProto};
use tract_onnx::prelude::TractResult;

const QUANTIZE: &str = "QuantizeLinear";
const DEQUANTIZE: &str = "DequantizeLinear";

pub(super) fn fold_constant_dequantization(proto: &mut ModelProto) -> TractResult<()> {
    let Some(graph) = proto.graph.as_mut() else {
        return Ok(());
    };
    fold_constants(graph)?;
    graph.node.iter_mut().for_each(forward_activation);
    Ok(())
}

fn forward_activation(node: &mut NodeProto) {
    if node.op_type == QUANTIZE || node.op_type == DEQUANTIZE {
        "Identity".clone_into(&mut node.op_type);
        node.input.truncate(1);
    }
}

fn constants_of(graph: &GraphProto) -> HashMap<String, TensorProto> {
    let named = graph.initializer.iter();
    named
        .map(|tensor| (tensor.name.clone(), tensor.clone()))
        .collect()
}

fn fold_constants(graph: &mut GraphProto) -> TractResult<()> {
    let constants = constants_of(graph);
    let folded = dequantized_constants(graph, &constants)?;
    graph
        .node
        .retain(|node| !is_constant_dequantization(node, &constants));
    graph.initializer.extend(folded);
    Ok(())
}

fn dequantized_constants(
    graph: &GraphProto,
    constants: &HashMap<String, TensorProto>,
) -> TractResult<Vec<TensorProto>> {
    let foldable = graph
        .node
        .iter()
        .filter(|node| is_constant_dequantization(node, constants));
    foldable
        .map(|node| tensor::dequantize(node, constants))
        .collect()
}

fn is_constant_dequantization(node: &NodeProto, constants: &HashMap<String, TensorProto>) -> bool {
    let operand = node.input.first();
    node.op_type == DEQUANTIZE && operand.is_some_and(|name| constants.contains_key(name))
}
