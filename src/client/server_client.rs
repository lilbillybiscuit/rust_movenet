
use opencv::core::{flip, Vec3b};
use opencv::videoio::*;
use opencv::{
    prelude::*,
    videoio,
    highgui::*,
};

use std::rc::Rc;

use tflitec::interpreter::{Interpreter, Options};
use tflitec::tensor::Tensor;
use tflitec::model::Model;
use prost::Message;
use crate::proto::DnnRequest;
use crate::proto::DnnResponse;
use crate::types::InferenceResults;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::TcpStream;
use std::io::{self, Write, Read};
use crate::utils::*;
use crate::types::*;

use log::{info, warn};
pub struct ServerClient {
    server_address: String,
    stream: TcpStream
}

impl ServerClient {
    // Initializes the ServerClient with the server address
    pub fn new(server_address: &str) -> Self {
        println!("Connecting to server at {}", server_address);
        let mut stream = TcpStream::connect(server_address).expect("Could not connect to server");
        println!("Connected!");

        ServerClient {
            server_address: server_address.to_string(),
            stream: stream
        }
    }

    fn connect_again(&mut self) {
        println!("Reconnecting to server at {}", self.server_address);
        self.stream = TcpStream::connect(&self.server_address).expect("Could not connect to server");
        println!("Connected!");
    }

    pub fn send_image_and_get_results(&mut self, image: &Mat) -> InferenceResults {
        let serialized_image= Image::from_mat(image);
        let serialized_image = resize_with_padding_ultra_fast(&serialized_image, (192, 192), "RGB");
        self.send_data_image(&serialized_image);
        self.receive_results()
    }

    pub fn send_data_image(&mut self, image: &Image) {
        self.send_data(&image.data, image.width as u32, image.height as u32);
    }
    pub fn send_data(&mut self, data: &Vec<u8>, width: u32, col: u32) {
        println!("Sending data to server...");
        println!("size of data: {}, width: {}, height: {}", data.len(), width, col);
        let mut data_yuv = vec![0; data.len()*2/3];
        rgb24_to_yuv422(data, &mut data_yuv);
        let data = data_yuv;

        let image_message = DnnRequest {
            image: data,
            width: width,
            height: col,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        };

        let mut buf = Vec::new();
        image_message.encode(&mut buf).unwrap();

        // get length of the message and send it
        let length = buf.len() as u32;
        let length_buffer = length.to_be_bytes();
        info!("Sent length of data: {}", length);
        self.stream.write_all(&length_buffer).expect("Failed to send length of data to server");

        // send the message
        self.stream.write_all(&buf).expect("Failed to send data to server");
        info!("Data sent!");
    }

    // Receives inference results from the server
    pub fn receive_results(&mut self) -> InferenceResults {
        let mut length_buffer = [0; 4];
        self.stream.read_exact(&mut length_buffer).expect("Failed to read length of response from server");
        let message_length = u32::from_be_bytes(length_buffer) as usize;

        let mut buf = vec![0; message_length];
        self.stream.read_exact(&mut buf).expect("Failed to read response from server");
        let response = DnnResponse::decode(&buf[..]).unwrap();

        InferenceResults {
            timestamp: response.timestamp,
            vector: response.vector.clone()
        }
    }
}

