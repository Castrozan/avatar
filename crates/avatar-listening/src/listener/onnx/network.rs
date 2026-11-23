use std::path::Path;

use tract_onnx::pb::ModelProto;
use tract_onnx::prelude::{
    Framework, InferenceFact, InferenceModelExt, RunnableModel, Symbol, SymbolValues, TDim, TValue,
    TVec, TractResult, TypedFact, TypedModel, TypedOp, onnx,
};

type Runnable = RunnableModel<TypedFact, Box<dyn TypedOp>, TypedModel>;
pub(super) type Rewrite = fn(&mut ModelProto) -> TractResult<()>;

#[derive(Debug)]
pub struct ModelLoadError(pub String);

pub struct Network {
    runnable: Runnable,
}

impl Network {
    pub fn load(
        path: &Path,
        input_facts: Vec<InferenceFact>,
        rewrite: Rewrite,
    ) -> Result<Self, ModelLoadError> {
        build(path, input_facts, rewrite)
            .map(|runnable| Self { runnable })
            .map_err(|error| ModelLoadError(format!("{error:?}")))
    }

    #[must_use]
    pub fn run(&self, inputs: TVec<TValue>) -> Option<TVec<TValue>> {
        self.runnable.run(inputs).ok()
    }
}

fn build(path: &Path, input_facts: Vec<InferenceFact>, rewrite: Rewrite) -> TractResult<Runnable> {
    let mut proto = onnx().proto_model_for_path(path)?;
    rewrite(&mut proto)?;
    let mut model = onnx().model_for_proto_model(&proto)?;
    for (index, fact) in input_facts.into_iter().enumerate() {
        model.set_input_fact(index, fact)?;
    }
    pin_batch_to_one(&model.into_typed()?)?
        .into_optimized()?
        .into_runnable()
}

fn pin_batch_to_one(model: &TypedModel) -> TractResult<TypedModel> {
    let shape = &model.input_fact(0)?.shape;
    let pinned = shape.iter().filter_map(symbol_of);
    let values = pinned.fold(SymbolValues::default(), |values, symbol| {
        values.with(&symbol, 1)
    });
    model.concretize_dims(&values)
}

fn symbol_of(dimension: &TDim) -> Option<Symbol> {
    let TDim::Sym(symbol) = dimension else {
        return None;
    };
    Some(symbol.clone())
}
