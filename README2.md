# Lab 2: DNN on the cloud


I used ChatGPT to plan out the steps for the lab, and then I mostly wrote the code myself.

Approximately ~30% (250 lines) of the code is written using ChatGPT, however it was not pure. Most of the generated code needed fixing due to borrowing issues in Rust.

## Part 3
For this I implemented my own Video for Linux API wrapper in Rust (located in the camera module), and optimized it to never copy anything to userspace through memory mapping (an exception to this is displaying the video, which it seems like is not the purpose of this assignment). I used a lot of ioctl commands in Rust, "translated" from similar uses of the Video for Linux API in C. I am getting nearly 40-50 frames per second (which I think is actually the limit of the camera on my Macbook).

To do this I had to refactor large parts of the code to support YUV and RGB interchangably (differences in processing 2 and 3 bytes respectively). I came up with algorithms quickly downscale images without relying on opencv, and on pure rust (and a package called rayon.)

I realized I made a slight mistake the on Part 2 by making server client too specific for the application. I have now modified it to use Protobuf to pass metadata, then a raw byte stream after it. I know this was mentioned as something I shouldn't do, but it was a necessary change and a useful one that proved to make future coding much more convenient.

In the future I will implement threading so we can achieve higher framerates, and we can skip the current sequential nature of the program that is causing major slowdowns.

To run this code, it is the same procedure as part 1+2.