use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use prost::Message;
use crate::proto::DnnRequest;
use crate::proto::DnnResponse;

use tflitec::interpreter::{Interpreter, Options};
use tflitec::tensor::Tensor;
use tflitec::model::Model;
use crate::types::InferenceResults;

pub fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:10026")?;
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

        // Read the full message based on the length
        if let Err(e) = stream.read_exact(&mut buffer[..message_length]) {
            eprintln!("Failed to read full message: {}", e);
            break;
        }

        let message = DnnRequest::decode(&buffer[..message_length]).expect("Failed to decode message");
        println!("Received Image timestamp: {}", message.timestamp);
    }
}
