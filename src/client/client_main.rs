use opencv::videoio::*;
use opencv::{
    prelude::*,
    videoio,
    highgui::*,
};
use structopt::StructOpt;
use client::app::App;
use client::server_client::ServerClient;
use crate::client;
use crate::types::Arguments;

use client::camera::Camera;

pub fn run_client() -> Result<(), Box<dyn std::error::Error>> {

    let opt = Arguments::from_args();
    
    // open camera
    // let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(); // 0 is the default camera
    // videoio::VideoCapture::is_opened(&cam).expect("Open camera [FAILED]");
    // cam.set(CAP_PROP_FPS, 30.0).expect("Set camera FPS [FAILED]");

    let mut cam = Camera::new(0);
    cam.set_frame_rate(30).expect("Set camera FPS [FAILED]");
    cam.start_capture().expect("Start camera capture [FAILED]");



    let server_client = ServerClient::new(opt.connect.as_str());
    let mut app = App::new(server_client, cam);


    loop {
        app.process_frame();
        let key = wait_key(1).unwrap();
        if key > 0 && key != 255 {
            break;
        }
    }
    Ok(())
}