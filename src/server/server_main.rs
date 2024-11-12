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
use crate::utils::{rgb24_to_yuv422, yuv422_to_rgb24};

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

fn inference(interpreter : &Interpreter, yuv_input: &[u8], original_res: (u32, u32)) -> Vec<f32> {

    assert!(yuv_input.len() %2 == 0, "YUV422 input size must be even");
    let mut rgb_input = vec![0; yuv_input.len() * 3/2];
    yuv422_to_rgb24(yuv_input, &mut rgb_input);
    interpreter.copy(&rgb_input[..], 0).unwrap();

    // interpreter.copy(&yuv_input[..], 0).unwrap();

    interpreter.invoke().expect("Invoke [FAILED]");

    let output_tensor = interpreter.output(0).unwrap();
    output_tensor.data::<f32>().to_vec()


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

        let time_start = std::time::Instant::now();

        info!("Expecting message of length: {}", message_length);

        // Read the full message based on the length
        if let Err(e) = stream.read_exact(&mut buffer[..message_length]) {
            eprintln!("Failed to read full message: {}", e);
            break;
        }

        // handle decoding of the message and processing it
        let message = DnnRequest::decode(&buffer[..message_length]).expect("Failed to decode message");
        info!("Received Image timestamp: {}", message.timestamp);
        let response = DnnResponse {
            timestamp: message.timestamp,
            vector: inference(&interpreter, &message.image, (message.width, message.height)),
        };

        // handle encoding of the response and sending it back
        let mut response_buffer = Vec::new();
        response.encode(&mut response_buffer).expect("Failed to encode response");
        let response_length = response_buffer.len() as u32;
        info!("Image {}: Sending response of length: {}", message.timestamp, response_length);
        stream.write_all(&response_length.to_be_bytes()).expect("Failed to write response length");
        stream.write_all(&response_buffer).expect("Failed to write response");

        let time_end = std::time::Instant::now();
        let elapsed = time_end - time_start;
        println!("Image {}: Inference took: {:?}", message.timestamp, elapsed);
    }
}
