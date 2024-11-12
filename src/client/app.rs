use std::slice;
use opencv::{
    prelude::*,
    highgui::*,
};

use crate::client::camera::Camera;
use crate::client::server_client::ServerClient;
use crate::types::COLOR_SPACE::{YUV};
use crate::types::{InferenceResults};
use crate::utils::{draw_keypoints};
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
        self.capture_image();
        let buffer_ptr = self.cam.get_buffer_ptr(); // Cast to a known type, e.g., u8
        let buffer_len = self.cam.get_buffer_length(); // Calculate the length

        // Create a slice from the raw pointer
        let buffer_slice = unsafe {
            assert!(!buffer_ptr.is_null()); // Ensure the pointer is not null
            slice::from_raw_parts(buffer_ptr, buffer_len)
        };

        self.server_client.send_data(&buffer_slice[..], 640, 480);
        let results = self.server_client.receive_results();

        let data_clone = buffer_slice.to_vec();
        let mut img = Image::new(data_clone, 640, 480, YUV);

        // let mut resized = resize_with_padding_ultra_fast(&img, (192, 192), YUV);
        let mut rgb_yuv_rgb = img.to_mat();


        self.display_results(&mut rgb_yuv_rgb, &results);

    }


    // Captures an image from the camera
    pub fn capture_image(&mut self) {
        self.cam.capture_image().expect("Capture image error");
    }

    // Displays the inference results on the captured image
    pub fn display_results(&self, frame: &mut Mat, results: &InferenceResults) {
        // Logic to draw keypoints on the image and display it
        draw_keypoints(frame, &results.vector[..], 0.25);
        imshow("MoveNet", frame).expect("imshow [ERROR]");
    }
}
