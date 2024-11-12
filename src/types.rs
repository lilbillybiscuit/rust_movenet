use std::time::{SystemTime, UNIX_EPOCH};
use opencv::core::{Mat, MatTraitConst, MatTraitConstManual};
use structopt::StructOpt;
use tflitec::tensor::Tensor;
pub struct InferenceResults {
    pub(crate) timestamp: u64,
    pub(crate) vector: Vec<f32>
}

pub struct Image {
    pub(crate) timestamp: u64,
    pub(crate) data: Vec<u8>,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Image {
    pub fn from_mat(image: &Mat) -> Image {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let bytes = image.data_bytes().unwrap();
        let mut data = Vec::with_capacity(bytes.len());
        data.extend_from_slice(bytes);

        Image {
            timestamp,
            data,
            width: image.rows() as i32,
            height: image.cols() as i32
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rust_movenet", about = "movenet in Rust")]
pub struct Arguments {
    #[structopt(short="s", long="server", help = "Run as server")]
    pub server: bool,

    #[structopt(short="c", long="client", help = "Run as client")]
    pub client: bool,

    #[structopt(short="m", long="main", help = "Run Main")]
    pub main: bool,

    #[structopt(short="bind", long="bind", default_value = "127.0.0.1:10026", help = "Bind address, only use for server")]
    pub bind: String,

    #[structopt(short="a", long="connect", default_value = "127.0.0.1:10026", help = "Connect address, only use for client")]
    pub connect: String,
}