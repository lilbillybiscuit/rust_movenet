use opencv::{
    prelude::*,
    videoio,
    highgui::*,
};

use opencv::core::{flip, Vec3b};

use crate::client::server_client::ServerClient;
use crate::types::InferenceResults;
use crate::utils::draw_keypoints;
use crate::utils::resize_with_padding;


pub struct App {
    server_client: ServerClient,
    cam: videoio::VideoCapture
}

impl App {
    /** Makes a new App struct. Must take in both a camera and a server client
    ** that are already initialized
    **/
    pub fn new(server_client: ServerClient, cam: videoio::VideoCapture) -> Self {

        App { server_client: server_client, cam: cam }
    }

    pub fn process_frame(&mut self) {
        let mut frame = self.capture_image();
        if frame.size().unwrap().width > 0 {
            // flip the image horizontally
            let mut flipped = Mat::default();
            flip(&frame, &mut flipped, 1).expect("flip [FAILED]");
            // resize the image as a square, size is
            let resized_img = resize_with_padding(&flipped, [192, 192]);

            let inference_results = {
                self.server_client.send_image_and_get_results(&resized_img)
            };
            self.display_results(&mut frame, &inference_results);
        }
    }


    fn capture_image(&mut self) -> Mat {
        // this function will also process the image somewhat
        let mut frame = Mat::default();
        self.cam.read(&mut frame).expect("VideoCapture: read [FAILED]");
        frame
    }

    // Displays the inference results on the captured image
    fn display_results(&self, frame: &mut Mat, results: &InferenceResults) {
        // Logic to draw keypoints on the image and display it
        draw_keypoints(frame, &results.vector[..], 0.25);
        imshow("MoveNet", frame).expect("imshow [ERROR]");
    }
}
