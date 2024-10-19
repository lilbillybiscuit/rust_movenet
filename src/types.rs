
use tflitec::tensor::Tensor;
pub struct InferenceResults<'a> {
    pub(crate) timestamp: u64,
    pub(crate) tensor: Tensor<'a>
}