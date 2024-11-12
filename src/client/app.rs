use opencv::{
    prelude::*,
    videoio,
    highgui::*,
};

use opencv::core::{flip, Vec3b, CV_8UC3};

use crate::client::server_client::ServerClient;
use crate::types::InferenceResults;
use crate::utils::{draw_keypoints, rgb24_to_yuv422, yuv422_to_rgb24};
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
            //
            // let mut rgb_yuv_rgb = {
            //     let mut vec_2d: Vec<Vec<Vec3b>> = flipped.to_vec_2d().unwrap();
            //     let mut vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();
            //     let length_1d = vec_1d.len();
            //     let mut rgb_yuv = vec![0; length_1d*2/3];
            //     rgb24_to_yuv422(&vec_1d, &mut rgb_yuv);
            //     let mut yuv_rgb = vec![0; length_1d];
            //     yuv422_to_rgb24(&rgb_yuv, &mut yuv_rgb);
            //     // overwrite vec2d
            //
            //     let rows = vec_2d.len();
            //     let cols = vec_2d[0].len();
            //     println!("Buffer length on yuv: {}", rgb_yuv.len());
            //     println!("Buffer length on new rgb: {}", yuv_rgb.len());
            //     println!("Width, height: {}, {}", cols, rows);
            //
            //     // convert to vector of Vec3b
            //     let mut vec_2d_rgb: Vec<Vec<Vec3b>> = vec![vec![Vec3b::default(); cols]; rows];
            //     for i in 0..rows {
            //         for j in 0..cols {
            //             let index = i * cols + j;
            //             vec_2d_rgb[i][j] = Vec3b::from_array(<[u8; 3]>::try_from(&yuv_rgb[index * 3..index * 3 + 3]).unwrap());
            //         }
            //     }
            //     Mat::from_slice_2d(&vec_2d_rgb).unwrap()
            //
            //
            // };

            self.display_results(&mut flipped, &inference_results);
        }
    }


    // Captures an image from the camera
    pub fn capture_image(&mut self) -> Mat {
        // this function will also process the image somewhat
        let mut frame = Mat::default();
        self.cam.read(&mut frame).expect("VideoCapture: read [FAILED]");
        frame
    }

    // Displays the inference results on the captured image
    pub fn display_results(&self, frame: &mut Mat, results: &InferenceResults) {
        // Logic to draw keypoints on the image and display it
        draw_keypoints(frame, &results.vector[..], 0.25);
        imshow("MoveNet", frame).expect("imshow [ERROR]");
    }
}
