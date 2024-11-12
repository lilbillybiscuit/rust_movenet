use opencv::{
    prelude::*,
    videoio,
    highgui::*,
};

use opencv::core::{flip, Vec3b, CV_8UC3};
use crate::client::camera::Camera;
use crate::client::server_client::ServerClient;
use crate::types::InferenceResults;
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
        let mut frame = self.capture_image();
        if frame.size().unwrap().width > 0 {
            // flip the image horizontally
            let mut flipped = Mat::default();
            flip(&frame, &mut flipped, 1).expect("flip [FAILED]");
            // resize the image as a square, size is

            // let resized_img = resize_with_padding(&flipped, [192, 192]);

            let inference_results = {
                self.server_client.send_image_and_get_results(&flipped)
            };

            let mut rgb_yuv_rgb = {
                let orig_image = Image::from_mat(&flipped);

                let img2 = resize_with_padding_ultra_fast(&orig_image, (192, 192) , "RGB",);

                let rows = img2.width as usize;
                let cols = img2.height as usize;
                println!("Buffer length on yuv: {}", img2.data.len());
                println!("Buffer length on new rgb: {}", img2.data.len());
                println!("Width, height: {}, {}", cols, rows);

                // convert to vector of Vec3b
                let mut vec_2d_rgb: Vec<Vec<Vec3b>> = vec![vec![Vec3b::default(); cols]; rows];
                for i in 0..rows {
                    for j in 0..cols {
                        let index = (i * cols + j) * 3;
                        vec_2d_rgb[i][j] = Vec3b::from_array([
                            img2.data[index],     // B
                            img2.data[index + 1],     // G
                            img2.data[index + 2]          // R
                        ]);
                    }
                }
                Mat::from_slice_2d(&vec_2d_rgb).unwrap()


            };

            self.display_results(&mut flipped, &inference_results);
        }
    }


    // Captures an image from the camera
    pub fn capture_image(&mut self) -> Mat {
        // this function will also process the image somewhat
        // let mut frame = Mat::default();
        let mut data = self.cam.capture_image().unwrap();
        let mut new_data = vec![0; data.len()*3/2];
        yuv422_to_rgb24(&data, &mut new_data);
        let mut rgb_yuv_rgb = {
            let rows = 480;
            let cols = 640;
            let mut vec_2d_rgb: Vec<Vec<Vec3b>> = vec![vec![Vec3b::default(); cols]; rows];
            for i in 0..rows {
                for j in 0..cols {
                    let index = (i * cols + j) * 3;
                    vec_2d_rgb[i][j] = Vec3b::from_array([
                        new_data[index],     // B
                        new_data[index + 1],     // G
                        new_data[index + 2]          // R
                    ]);
                }
            }
            Mat::from_slice_2d(&vec_2d_rgb).unwrap()
        };
        rgb_yuv_rgb
    }

    // Displays the inference results on the captured image
    pub fn display_results(&self, frame: &mut Mat, results: &InferenceResults) {
        // Logic to draw keypoints on the image and display it
        draw_keypoints(frame, &results.vector[..], 0.25);
        imshow("MoveNet", frame).expect("imshow [ERROR]");
    }
}
