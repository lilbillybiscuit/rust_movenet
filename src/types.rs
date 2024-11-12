use std::cmp::PartialEq;
use std::time::{SystemTime, UNIX_EPOCH};
use opencv::core::{Mat, MatTraitConst, MatTraitConstManual, Vec3b};
use structopt::StructOpt;
use tflitec::tensor::Tensor;
use crate::types::COLOR_SPACE::RGB;
use crate::utils::yuv422_to_rgb24;

pub struct InferenceResults {
    pub(crate) timestamp: u64,
    pub(crate) vector: Vec<f32>
}

pub enum COLOR_SPACE {
    RGB,
    YUV
}
pub struct Image {
    pub(crate) timestamp: u64,
    pub(crate) data: Vec<u8>,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) color_space: COLOR_SPACE // either RGB or YUV
}

impl Image {

    pub fn new(data: Vec<u8>, width: i32, height: i32, color_space: COLOR_SPACE) -> Image {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Image {
            timestamp,
            data,
            width,
            height,
            color_space
        }
    }
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
            width: image.cols() as i32,
            height: image.rows() as i32,
            color_space: RGB
        }
    }

    pub fn flip(&mut self) {
        let width = self.width as usize;
        let height = self.height as usize;
        let bytes_per_pixel = 3;

        for y in 0..height {
            let start_index = y * width * bytes_per_pixel;
            let end_index = start_index + width * bytes_per_pixel;

            let row = &mut self.data[start_index..end_index];

            row.chunks_exact_mut(bytes_per_pixel).for_each(|chunk| chunk.reverse());
        }
    }

    pub fn to_mat(&self) -> Mat {


        let thisimage:&Image = match &self.color_space {
            COLOR_SPACE::YUV => {
                let mut img2_rgb = vec![0; self.width as usize * self.height as usize * 3];
                yuv422_to_rgb24(&self.data, &mut img2_rgb);
                &Image {
                    timestamp: 0,
                    data: img2_rgb,
                    width: self.width,
                    height: self.height,
                    color_space: RGB
                }
            },
            COLOR_SPACE::RGB => self
        };
        let rows = thisimage.height as usize;
        let cols = thisimage.width as usize;

        let mut vec_2d_rgb: Vec<Vec<Vec3b>> = vec![vec![Vec3b::default(); cols]; rows];
        for i in 0..rows {
            for j in 0..cols {
                let index = (i * cols + j) * 3;
                vec_2d_rgb[i][j] = Vec3b::from_array([
                    thisimage.data[index],     // B
                    thisimage.data[index + 1],     // G
                    thisimage.data[index + 2]          // R
                ]);
            }
        }
        Mat::from_slice_2d(&vec_2d_rgb).unwrap()
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