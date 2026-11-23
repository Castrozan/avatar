use tract_onnx::pb::{TensorProto, tensor_proto::DataType};

fn words(raw: &[u8]) -> impl Iterator<Item = [u8; 4]> + '_ {
    raw.chunks_exact(4)
        .filter_map(|chunk| <[u8; 4]>::try_from(chunk).ok())
}

pub(super) fn floats(tensor: &TensorProto) -> Vec<f32> {
    if tensor.raw_data.is_empty() {
        return tensor.float_data.clone();
    }
    words(&tensor.raw_data).map(f32::from_le_bytes).collect()
}

pub(super) fn integers(tensor: &TensorProto) -> Vec<i32> {
    if tensor.raw_data.is_empty() {
        return tensor.int32_data.clone();
    }
    raw_integers(DataType::from_i32(tensor.data_type), &tensor.raw_data)
}

fn raw_integers(data_type: Option<DataType>, raw: &[u8]) -> Vec<i32> {
    match data_type {
        Some(DataType::Int8) => raw.iter().map(|&byte| signed(byte)).collect(),
        Some(DataType::Uint8) => raw.iter().map(|&byte| i32::from(byte)).collect(),
        _ => words(raw).map(i32::from_le_bytes).collect(),
    }
}

fn signed(byte: u8) -> i32 {
    i32::from(i8::from_le_bytes([byte]))
}
