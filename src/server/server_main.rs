use std::fmt::Debug;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use prost::Message;
use structopt::StructOpt;
use crate::proto::DnnRequest;
use crate::proto::DnnResponse;

use tflitec::interpreter::{Interpreter, Options};
use tflitec::tensor::Tensor;
use tflitec::model::Model;
use crate::types::{InferenceResults, Arguments};

use log::{info, warn};


pub fn run_server() -> std::io::Result<()> {
    let opt = Arguments::from_args();
    let listener = TcpListener::bind(opt.bind)?;
    println!("Server listening on port 10026");


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to accept a connection: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = vec![0; 1024];

    let options = Options::default();
    let path = "resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite";
    let model = Model::new(&path).expect("Load model [FAILED]");
    let interpreter = Interpreter::new(&model, Some(options)).expect("Create interpreter [FAILED]");
    interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");


    loop {
        let mut length_buffer = [0; 4]; // let length be 4-byte usize
        if let Err(e) = stream.read_exact(&mut length_buffer) {
            eprintln!("Failed to read message length: {}", e);
            return;
        }

        let message_length = u32::from_be_bytes(length_buffer) as usize;

        // Ensure buffer is large enough to hold the incoming message
        if buffer.len() < message_length {
            buffer.resize(message_length, 0);
        }

        info!("Expecting message of length: {}", message_length);

        // Read the full message based on the length
        if let Err(e) = stream.read_exact(&mut buffer[..message_length]) {
            eprintln!("Failed to read full message: {}", e);
            break;
        }

        // handle decoding of the message and processing it
        let message = DnnRequest::decode(&buffer[..message_length]).expect("Failed to decode message");
        info!("Received Image timestamp: {}", message.timestamp);
        interpreter.copy(&message.image[..], 0).unwrap();
        interpreter.invoke().expect("Invoke [FAILED]");

        let output_tensor = interpreter.output(0).unwrap();
        let response = DnnResponse {
            timestamp: message.timestamp,
            vector: output_tensor.data::<f32>().to_vec(),
        };

        // handle encoding of the response and sending it back
        let mut response_buffer = Vec::new();
        response.encode(&mut response_buffer).expect("Failed to encode response");
        let response_length = response_buffer.len() as u32;
        info!("Image {}: Sending response of length: {}", message.timestamp, response_length);
        stream.write_all(&response_length.to_be_bytes()).expect("Failed to write response length");
        stream.write_all(&response_buffer).expect("Failed to write response");
    }
}
