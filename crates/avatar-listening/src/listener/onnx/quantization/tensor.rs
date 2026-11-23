use std::collections::HashMap;

use num_traits::AsPrimitive;
use tract_onnx::pb::{NodeProto, TensorProto, tensor_proto::DataType};
use tract_onnx::prelude::TractResult;
use tract_onnx::tract_core::internal::bail;

use super::decode::{floats, integers};

const DEFAULT_AXIS: i64 = 1;

struct Layout {
    stride: usize,
    channels: usize,
}

struct Parameters {
    scales: Vec<f32>,
    zeros: Vec<i32>,
}

struct Operands<'tensors> {
    quantized: &'tensors TensorProto,
    parameters: Parameters,
}

fn operands<'tensors>(
    node: &NodeProto,
    constants: &'tensors HashMap<String, TensorProto>,
) -> TractResult<Operands<'tensors>> {
    let find = |slot: usize| node.input.get(slot).and_then(|name| constants.get(name));
    let (Some(quantized), Some(scale)) = (find(0), find(1)) else {
        bail!("dequantization of {} lacks constant operands", node.name);
    };
    Ok(Operands {
        quantized,
        parameters: Parameters::new(floats(scale), find(2).map_or_else(Vec::new, integers)),
    })
}

impl Parameters {
    fn new(scales: Vec<f32>, zeros: Vec<i32>) -> Self {
        Self { scales, zeros }
    }
}

pub(super) fn dequantize(
    node: &NodeProto,
    constants: &HashMap<String, TensorProto>,
) -> TractResult<TensorProto> {
    let operands = operands(node, constants)?;
    let layout = layout_of(node, &operands.quantized.dims);
    let levels = integers(operands.quantized).into_iter().enumerate();
    let values = levels.map(|(index, level)| {
        let channel = (index / layout.stride) % layout.channels;
        value_of(level, channel, &operands.parameters)
    });
    Ok(float_tensor(node, operands.quantized, values))
}

fn float_tensor(
    node: &NodeProto,
    quantized: &TensorProto,
    values: impl Iterator<Item = f32>,
) -> TensorProto {
    TensorProto {
        name: node.output[0].clone(),
        dims: quantized.dims.clone(),
        data_type: DataType::Float as i32,
        float_data: values.collect(),
        ..TensorProto::default()
    }
}

fn value_of(level: i32, channel: usize, parameters: &Parameters) -> f32 {
    let zero = parameters.zeros.get(pick(parameters.zeros.len(), channel));
    let steps: f32 = (level - zero.copied().unwrap_or(0)).as_();
    steps * parameters.scales[pick(parameters.scales.len(), channel)]
}

fn pick(length: usize, channel: usize) -> usize {
    if length == 1 { 0 } else { channel }
}

fn axis_of(node: &NodeProto, rank: usize) -> usize {
    let declared = node
        .attribute
        .iter()
        .find(|attribute| attribute.name == "axis")
        .map_or(DEFAULT_AXIS, |attribute| attribute.i);
    let rank = i64::try_from(rank).unwrap_or(1).max(1);
    usize::try_from(declared.rem_euclid(rank)).unwrap_or(0)
}

fn layout_of(node: &NodeProto, dims: &[i64]) -> Layout {
    let axis = axis_of(node, dims.len());
    let sizes: Vec<usize> = dims
        .iter()
        .map(|&size| usize::try_from(size).unwrap_or(1).max(1))
        .collect();
    Layout {
        stride: sizes[axis + 1..].iter().product(),
        channels: sizes[axis],
    }
}
