use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};
use opencv::{
    prelude::*,
    videoio,
    highgui::*,
};

use opencv::core::{flip, Vec3b, CV_8UC3};
use crate::client::camera::Camera;
use crate::client::server_client::ServerClient;
use crate::types::COLOR_SPACE::{RGB, YUV};
use crate::types::{ImageBuffer, InferenceResults, COLOR_SPACE};
use crate::utils::{draw_keypoints, resize_with_padding_ultra_fast, rgb24_to_yuv422, yuv422_to_rgb24};
use crate::utils::resize_with_padding;
use crate::types::Image;


pub struct App {
    server_client: ServerClient,
    cam: Camera
}

impl App {
    /** Makes a new App struct. Must take in both a camera and a server client
    ** that are already initialized
    **/
    pub fn new(server_client: ServerClient, cam: Camera) -> Self {

        App { server_client: server_client, cam: cam }
    }
    
    // Processes a frame from the camera, the entire pipeline
    pub fn process_frame(&mut self) {
        let frame = self.capture_image();
        self.server_client.send_data(&self.cam.buffer[..], frame.width as u32, frame.height as u32);
        let mut results = self.server_client.receive_results();

        // let mut rgb_yuv_rgb = frame.to_mat();
        //
        // self.display_results(&mut rgb_yuv_rgb, &results);

    }


    // Captures an image from the camera
    pub fn capture_image(&mut self) -> ImageBuffer {
        self.cam.capture_image().expect("Capture image error");

        let buffer_ptr = self.cam.buffer.buffer as *const u8; // Cast to a known type, e.g., u8
        let buffer_len = self.cam.buffer.length; // Calculate the length

        // Create a slice from the raw pointer
        let buffer_slice = unsafe {
            assert!(!buffer_ptr.is_null()); // Ensure the pointer is not null
            slice::from_raw_parts(buffer_ptr, buffer_len)
        };
        ImageBuffer {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            buffer: buffer_slice,
            width: 640,
            height: 480,
            color_space: COLOR_SPACE::YUV,
            length: buffer_len as i32,
        }
    }

    // Displays the inference results on the captured image
    pub fn display_results(&self, frame: &mut Mat, results: &InferenceResults) {
        // Logic to draw keypoints on the image and display it
        draw_keypoints(frame, &results.vector[..], 0.25);
        imshow("MoveNet", frame).expect("imshow [ERROR]");
    }
}
