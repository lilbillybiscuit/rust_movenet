
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
        let (serialized_image, width, col) = self.serialize_image(image);
        self.send_data(&serialized_image, width, col);
        self.receive_results()
    }

    fn serialize_image(&self, image: &Mat) -> (Vec<u8>, u32, u32) {
        let vec_2d: Vec<Vec<Vec3b>> = image.to_vec_2d().unwrap();
        let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();
        (vec_1d, image.rows() as u32, image.cols() as u32)
    }

    fn send_data(&mut self, data: &Vec<u8>, width: u32, col: u32) {
        println!("Sending data to server...");
        println!("size of data: {}", data.len());
        let image_message = DnnRequest {
            image: data.clone(),
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
    fn receive_results(&mut self) -> InferenceResults {
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

