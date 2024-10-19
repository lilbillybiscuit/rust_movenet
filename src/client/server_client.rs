
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
use crate::types::InferenceResults;

use std::time::{SystemTime, UNIX_EPOCH};
pub struct ServerClient<'a> {
    server_address: String,
    interpreter: Interpreter<'a>,
    model: Rc<Model<'a>>,
}

impl<'interp> ServerClient<'interp> {
    // Initializes the ServerClient with the server address
    pub fn new(server_address: &str) -> Self {
        // Load model and create interpreter


        ServerClient {
            server_address: server_address.to_string(),
            interpreter,
            model, // Store the model in the struct
            // Initialize network connection here
        }
    }

    pub fn send_image_and_get_results(&self, image: &Mat) -> InferenceResults {
        let serialized_image = self.serialize_image(image);
        self.send_data(&serialized_image);
        self.receive_results()
    }

    fn serialize_image(&self, image: &Mat) -> Vec<u8> {
        let vec_2d: Vec<Vec<Vec3b>> = image.to_vec_2d().unwrap();
        let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();
        vec_1d
    }

    fn send_data(&self, data: &Vec<u8>) {
        println!("Sending data to server...");
        println!("size of data: {}", data.len());
        // set input (tensor0)
        self.interpreter.copy(&data[..], 0).unwrap();
        self.interpreter.invoke().expect("Invoke [FAILED]");
    }

    // Receives inference results from the server
    fn receive_results(&self) -> InferenceResults {
        let output_tensor = self.interpreter.output(0).unwrap();
        InferenceResults {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            tensor: output_tensor
        }
    }
}

