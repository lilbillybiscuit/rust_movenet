use structopt::StructOpt;
use tflitec::tensor::Tensor;
pub struct InferenceResults {
    pub(crate) timestamp: u64,
    pub(crate) vector: Vec<f32>
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rust_movenet", about = "movenet in Rust")]
pub struct Arguments {
    #[structopt(short="s", long="server", help = "Run as server")]
    pub server: bool,

    #[structopt(short="c", long="client", help = "Run as client")]
    pub client: bool,

    #[structopt(short="bind", long="bind", default_value = "127.0.0.1:10026", help = "Bind address, only use for server")]
    pub bind: String,

    #[structopt(short="a", long="connect", default_value = "127.0.0.1:10026", help = "Connect address, only use for client")]
    pub connect: String,
}