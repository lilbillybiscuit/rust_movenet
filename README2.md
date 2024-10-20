# Lab 2: DNN on the cloud


I used ChatGPT to plan out the steps for the lab, and then I mostly wrote the code myself.

Approximately ~30% (130 lines) of the code is written using ChatGPT, however it was not pure. Most of the generated code needed fixing due to borrowing issues in Rust.

## Part 1
This step was relatively straightforward, Parallels Desktop handled everything

## Part 2

**To run:**

```shell
# To start client
cargo run -r -- -c --addr <address>

# To start server
cargo run -r -- -s --bind <address>

```

I would like to describe my implementation and philosophy for my DNN program.

First, I set out to design a simple but still controllable App interface. While I wanted the App class to do most of the heavy lifting, I still wanted to provide a level of customizability to the user. Therefore, we make the User specify the camera and the server client. This way, the user could swap out the camera, or swap out the server client without having to change the App class, and still utilize similar functionality.

The interface looks as follows:

```rust
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
        ...
    }

    pub fn capture_image(&mut self) -> Mat { // allows for some debugging/custom outputs
        ...
    }
    
    pub fn display_results(&self, frame: &mut Mat, results: &InferenceResults) {
        ...
    }
}
```

To start and run the app, we just need the following code to keep calling process_frame:

```rust
fn run_client() {
    // create a camera
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(); // 0 is the default camera
    videoio::VideoCapture::is_opened(&cam).expect("Open camera [FAILED]");
    cam.set(CAP_PROP_FPS, 30.0).expect("Set camera FPS [FAILED]");

    // create a server
    let server_client = ServerClient::new(opt.connect.as_str());
    
    let mut app = App::new(server_client, cam);
    // loop the app
    loop {
        app.process_frame();
        //...
    }
}

```

Therefore, the interface has been simplified to the user, but is still powerful enough to be customizable.

Now the App takes in a server client. We also provide a relatively simple server client interface that also achieves super high performance. The server interface looks as follows:

```rust
impl ServerClient {
    // Initializes the ServerClient with the server address
    pub fn new(server_address: &str) -> Self {
        //...
    }

    pub fn send_image_and_get_results(&mut self, image: &Mat) -> InferenceResults {
        //...
    }

    pub fn send_data(&mut self, data: &Vec<u8>, width: u32, col: u32) {
        //...
    }

    // Receives inference results from the server
    pub fn receive_results(&mut self) -> InferenceResults {
        //...
    }
}
```

The app will call send_image_and_get_results to recieve the data. For communicating between the server and client, I tried to reduce as much overhead as possible, from preventing copies, reducing protocol overhead and ensuring correctness, so I used the Rust implementation of protobuf called prost. This allowed me to serialize and deserialize the data in a very efficient manner. The protobuf message essentially contains a byte stream of the image, and the server returns a float "array" of the results.

The server takes these, decodes, them, runs them through the interpreter, then sends the results back to the client. The server is implemented in Python, and the client is implemented in Rust. The server is a simple Flask server that takes in the image, runs it through the interpreter, and sends the results back. To handle concurrent requests just in case, the server allocates a new thread for each stream. Since the methods use TCP (in std::net), we can mostly assume in-order delivery and correctness of transferred data since TCP will its own checksumming/verification process.