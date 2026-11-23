mod evaluation;

use tract_onnx::pb::{GraphProto, ModelProto, NodeProto};
use tract_onnx::prelude::{TValue, TractResult};

const CONDITIONAL: &str = "If";

pub(super) fn resolve_conditionals(proto: &mut ModelProto, probe: &[TValue]) -> TractResult<()> {
    while let Some(position) = first_conditional(proto) {
        let taken = evaluation::condition_holds(proto, position, probe)?;
        inline_branch(proto, position, taken);
    }
    Ok(())
}

fn first_conditional(proto: &ModelProto) -> Option<usize> {
    let graph = proto.graph.as_ref()?;
    graph
        .node
        .iter()
        .position(|node| node.op_type == CONDITIONAL)
}

fn inline_branch(proto: &mut ModelProto, position: usize, taken: bool) {
    let Some(graph) = proto.graph.as_mut() else {
        return;
    };
    let conditional = graph.node.remove(position);
    let branch = branch_of(&conditional, taken);
    splice(graph, position, &conditional, branch);
}

fn branch_of(conditional: &NodeProto, taken: bool) -> GraphProto {
    let wanted = if taken { "then_branch" } else { "else_branch" };
    conditional
        .attribute
        .iter()
        .find(|attribute| attribute.name == wanted)
        .and_then(|attribute| attribute.g.clone())
        .unwrap_or_default()
}

fn forward(produced: &str, expected: &str) -> NodeProto {
    NodeProto {
        op_type: "Identity".to_owned(),
        input: vec![produced.to_owned()],
        output: vec![expected.to_owned()],
        ..NodeProto::default()
    }
}

fn splice(graph: &mut GraphProto, position: usize, conditional: &NodeProto, branch: GraphProto) {
    let forwards: Vec<NodeProto> = branch
        .output
        .iter()
        .zip(&conditional.output)
        .map(|(produced, expected)| forward(&produced.name, expected))
        .collect();
    graph.initializer.extend(branch.initializer);
    graph
        .node
        .splice(position..position, branch.node.into_iter().chain(forwards));
}
