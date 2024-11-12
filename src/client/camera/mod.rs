mod bindings;

use crate::client::camera::bindings::*;

use std::{fs::File, os::unix::prelude::AsRawFd, str};
use nix::{sys::ioctl, ioctl_read, ioctl_readwrite, ioctl_write_ptr};
use std::mem::size_of;
use std::os::fd::RawFd;
use nix::libc::{ioctl, KERN_ACPI_VIDEO_FLAGS};

use log::{info, warn, error};

use libc::mmap;
// #define VIDIOC_QUERYCAP		 _IOR('V',  0, struct v4l2_capability)

const VIDIOC_QUERYCAP_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYCAP_TYPE_MODE: u8 = 0;

struct MmappedBuffer {
    pub buffer: *mut libc::c_void,
    pub length: usize
}

pub struct Camera {
    file: File,
    pub buffer: MmappedBuffer,
    fps: i32,
    streaming: bool,
}

impl Camera {
    pub fn new(index: u32) -> Self {

        // C: open camera device
        let mut file = File::options()
            .write(true)
            .read(true)
            .open(format!("/dev/video{}", index))
            .unwrap();
        let media_fd = file.as_raw_fd();
        info!("camera fd = {}", media_fd);

        if media_fd < 0 {
            panic!("failed to open camera device");
        }

        let init_device_result = Camera::init_device(&media_fd);
        match init_device_result {
            Ok(fmt) => {
                info!("init device [OK]");
            }
            Err(e) => {
                panic!("init device [FAILED]: {:?}", e);
            }
        }

        let init_buffer_result = Camera::allocate_and_mmap(&media_fd);
        let buffer: MmappedBuffer;
        match init_buffer_result {
            Ok(inner_buffer) => {
                info!("allocate and mmap [OK]");
                buffer = inner_buffer;
            }
            Err(e) => {
                panic!("allocate and mmap [FAILED]: {:?}", e);
            }
        }


        Camera {
            file: file,
            buffer: buffer,
            fps: 0,
            streaming: false,
        }

    }

    pub fn init_device(media_fd1: &RawFd) -> Result<bool, String> {
        let media_fd = *media_fd1;
        // 2: query capabilities
        let mut info: v4l2_capability = unsafe { std::mem::zeroed() };
        ioctl_read!(vidioc_querycap, VIDIOC_QUERYCAP_MAGIC, VIDIOC_QUERYCAP_TYPE_MODE, v4l2_capability);
        match unsafe { vidioc_querycap(media_fd, &mut info) } {
            Ok(_) => {
                info!("IOCTL: get info querycap [OK]");
                info!("driver: {:?}", str::from_utf8(&info.driver));
                info!("card: {:?}", str::from_utf8(&info.card));
                info!("bus_info: {:?}", str::from_utf8(&info.bus_info));
                info!("version: {:?}", info.version);
                info!("capabilities: {:x}", info.capabilities);
                info!("device_caps: {:?}", info.device_caps);
            }
            Err(e) => {
                return Err("get info querycap [FAILED]".to_string());
            }
        }

        // 3: check video input
        // #define VIDIOC_G_INPUT           _IOR('V', 38, int)
        let mut input_index: u32 = 0;
        ioctl_read!(vidioc_g_input, VIDIOC_QUERYCAP_MAGIC, 38, u32); // for some reason bindgen treats this as u32

        match unsafe { vidioc_g_input(media_fd, &mut input_index) } {
            Ok(_) => {
                info!("IOCTL: get input index [OK]: {}", input_index);
            }
            Err(e) => {
                return Err("get input index [FAILED]".to_string());
            }
        }

        // #define VIDIOC_ENUMINPUT	_IOWR('V', 26, struct v4l2_input)
        let mut input: v4l2_input = unsafe { std::mem::zeroed() };
        ioctl_readwrite!(vidioc_enum_input, VIDIOC_QUERYCAP_MAGIC, 26, v4l2_input);
        input.index = input_index;

        match unsafe { vidioc_enum_input(media_fd, &mut input) } {
            Ok(_) => {
                info!("IOCTL: get input [OK]");
                info!("name: {:?}", str::from_utf8(&input.name));
                info!("type: {:?}", input.type_);
                info!("status: {:?}", input.status);
                info!("capabilities: {:?}", input.capabilities);
            }
            Err(e) => {
                info!("get input [FAILED]: {:?}", e);
                return Err("get input [FAILED]".to_string());
            }
        }


        // 4: check image format
        // #define VIDIOC_G_FMT		_IOWR('V', 4, struct v4l2_format)
        let mut fmt: v4l2_format = unsafe { std::mem::zeroed() };
        ioctl_readwrite!(vidioc_g_fmt, VIDIOC_QUERYCAP_MAGIC, 4, v4l2_format);
        fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;

        match unsafe { vidioc_g_fmt(media_fd, &mut fmt) } {
            Ok(_) => {
                info!("IOCTL: get format [OK]");
                unsafe {
                    info!("width: {:?}", fmt.fmt.pix.width);
                    info!("height: {:?}", fmt.fmt.pix.height);
                    info!("pixelformat: {:?}", fmt.fmt.pix.pixelformat);
                    info!("field: {:?}", fmt.fmt.pix.field);
                    info!("bytesperline: {:?}", fmt.fmt.pix.bytesperline);
                    info!("sizeimage: {:?}", fmt.fmt.pix.sizeimage);
                    info!("colorspace: {:?}", fmt.fmt.pix.colorspace);
                    info!("priv: {:?}", fmt.fmt.pix.priv_);
                }
            }
            Err(e) => {
                return Err("get format [FAILED]".to_string());
            }
        }

        // C: check supported image format
        let mut fmtdesc: v4l2_fmtdesc = unsafe { std::mem::zeroed() };
        fmtdesc.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        fmtdesc.index = 0;
        info!("===Getting supported format===");
        // #define VIDIOC_ENUM_FMT         _IOWR('V',  2, struct v4l2_fmtdesc)
        ioctl_readwrite!(vidioc_enum_fmt, VIDIOC_QUERYCAP_MAGIC, 2, v4l2_fmtdesc);

        loop {
            match unsafe { vidioc_enum_fmt(media_fd, &mut fmtdesc) } {
                Ok(_) => {
                    info!("IOCTL: get supported format [OK]");
                    info!("index: {:?}", fmtdesc.index);
                    info!("pixelformat: {:?}", fmtdesc.pixelformat);
                    info!("description: {:?}", str::from_utf8(&fmtdesc.description));
                }
                Err(e) => {
                    break;
                }
            }
            fmtdesc.index += 1;
        }

        // switch to YUYV format
        // #define VIDIOC_S_FMT		_IOWR('V', 5, struct v4l2_format)
        // format.fmt.pix.pixelformat = 0x56595559; // 'V', 'Y', 'U', 'Y'
        unsafe {
            fmt.fmt.pix.pixelformat = 0x56595559; // 'V', 'Y', 'U', 'Y'
            fmt.fmt.pix.width = 192;
            fmt.fmt.pix.height = 192;
        }
        ioctl_readwrite!(vidio_s_fmt, VIDIOC_QUERYCAP_MAGIC, 5, v4l2_format);
        match unsafe { vidio_s_fmt(media_fd, &mut fmt) } {
            Ok(_) => {
                info!("IOCTL: set format [OK]");
            }
            Err(e) => {
                return Err("set format [FAILED]".to_string());
            }
        }

        match unsafe {vidioc_g_fmt(media_fd, &mut fmt)} {
            Ok(_) => {
                info!("IOCTL: get format [OK]");
                unsafe {
                    info!("width: {:?}", fmt.fmt.pix.width);
                    info!("height: {:?}", fmt.fmt.pix.height);
                    info!("pixelformat: {:?}", fmt.fmt.pix.pixelformat);
                    info!("field: {:?}", fmt.fmt.pix.field);
                    info!("bytesperline: {:?}", fmt.fmt.pix.bytesperline);
                    info!("sizeimage: {:?}", fmt.fmt.pix.sizeimage);
                    info!("colorspace: {:?}", fmt.fmt.pix.colorspace);
                    info!("priv: {:?}", fmt.fmt.pix.priv_);
                }
            }
            Err(e) => {
                return Err("get format [FAILED]".to_string());
            }
        }

        return Ok(true);
    }
    pub fn allocate_and_mmap(media_fd1: &RawFd) -> Result<MmappedBuffer, String> {
        let media_fd = *media_fd1;
        // 1: request and allocate buffers
        let mut reqbufs: v4l2_requestbuffers = unsafe { std::mem::zeroed() };
        reqbufs.count = 4;
        reqbufs.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        reqbufs.memory = v4l2_memory_V4L2_MEMORY_MMAP;

        ioctl_readwrite!(vidioc_reqbufs, VIDIOC_QUERYCAP_MAGIC, 8, v4l2_requestbuffers); // #define VIDIOC_REQBUFS		_IOWR('V', 8, struct v4l2_requestbuffers)
        match unsafe { vidioc_reqbufs(media_fd, &mut reqbufs) } {
            Ok(_) => {
                info!("IOCTL: request buffer [OK]");
                info!("count: {:?}", reqbufs.count);
                info!("type: {:?}", reqbufs.type_);
                info!("memory: {:?}", reqbufs.memory);
            }
            Err(e) => {
                return Err("request buffer [FAILED]".to_string());
            }
        }

        // 2: get details of the allocated buffer and map it
        // #define VIDIOC_QUERYBUF		_IOWR('V', 9, struct v4l2_buffer)
        let mut buf: v4l2_buffer = unsafe { std::mem::zeroed() };
        buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
        buf.index = 0;

        ioctl_readwrite!(vidioc_querybuf, VIDIOC_QUERYCAP_MAGIC, 9, v4l2_buffer);
        match unsafe { vidioc_querybuf(media_fd, &mut buf) } {
            Ok(_) => {
                info!("IOCTL: query buffer [OK]");
                info!("index: {:?}", buf.index);
                info!("type: {:?}", buf.type_);
                info!("bytesused: {:?}", buf.bytesused);
                info!("flags: {:?}", buf.flags);
                info!("field: {:?}", buf.field);
                info!("timestamp: {:?}", buf.timestamp);
                info!("timecode: {:?}", buf.timecode);
                info!("sequence: {:?}", buf.sequence);
                info!("memory: {:?}", buf.memory);
                // info!("offset: {:?}", buf.m.offset);
                info!("length: {:?}", buf.length);
            }
            Err(e) => {
                return Err("query buffer [FAILED]".to_string());
            }
        }

        // map buffer now
        let mut buffer = unsafe {
            mmap(std::ptr::null_mut(),
                 buf.length as usize,
                 libc::PROT_READ | libc::PROT_WRITE,
                 libc::MAP_SHARED,
                 media_fd, buf.m.offset as libc::off_t)
        };

        if buffer == libc::MAP_FAILED {
            return Err("mmap failed".to_string());
        }

        Ok(MmappedBuffer {
            buffer,
            length: buf.length as usize
        })
    }

    pub fn set_frame_rate(&mut self, new_fps: i32) -> Result<bool, String> {
        let media_fd = self.file.as_raw_fd();
        // #define VIDIOC_S_PARM		_IOWR('V', 22, struct v4l2_streamparm)
        let mut parm: v4l2_streamparm = unsafe { std::mem::zeroed() };

        unsafe {
            parm.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            parm.parm.capture.timeperframe.denominator = new_fps as u32;
            parm.parm.capture.timeperframe.numerator = 1;
            parm.parm.capture.capability = V4L2_CAP_TIMEPERFRAME;
            parm.parm.output.timeperframe.denominator = new_fps as u32;
            parm.parm.output.timeperframe.numerator = 1;
            parm.parm.output.capability = V4L2_CAP_TIMEPERFRAME;
        }

        ioctl_readwrite!(vidioc_s_parm, VIDIOC_QUERYCAP_MAGIC, 22, v4l2_streamparm);
        match unsafe { vidioc_s_parm(media_fd, &mut parm) } {
            Ok(_) => {
                info!("IOCTL: set frame rate [OK]");
                self.fps = new_fps;
                return Ok(true);
            }
            Err(e) => {
                return Err(format!("set frame rate [FAILED]: {:?}", e));
            }
        }
    }

    pub fn start_capture(&self) -> Result<bool, String> {
        if self.fps == 0 {
            return Err("frame rate not set".to_string());
        }
        if self.streaming {
            return Ok(true);
        }
        let media_fd = self.file.as_raw_fd();
        // #define VIDIOC_STREAMON		_IOW('V', 18, int)
        ioctl_write_ptr!(vidioc_streamon, VIDIOC_QUERYCAP_MAGIC, 18, i32);
        let mut stream_type = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE as i32;
        match unsafe { vidioc_streamon(media_fd, &mut stream_type) } {
            Ok(_) => {
                info!("IOCTL: stream on [OK]");
            }
            Err(e) => {
                return Err("stream on [FAILED]".to_string());
            }
        }
        self.enqueue_buffer();
        Ok(true)
    }

    pub fn stop_capture(&self) -> Result<bool, String> {
        if !self.streaming {
            return Ok(true);
        }
        let media_fd = self.file.as_raw_fd();

        // #define VIDIOC_STREAMOFF	_IOW('V', 19, int)
        ioctl_write_ptr!(vidioc_streamoff, VIDIOC_QUERYCAP_MAGIC, 19, i32);
        let mut stream_type = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE as i32;
        match unsafe { vidioc_streamoff(media_fd, &mut stream_type) } {
            Ok(_) => {
                info!("IOCTL: stream off [OK]");
            }
            Err(e) => {
                return Err("stream off [FAILED]".to_string());
            }
        }
        Ok(true)
    }
    // to capture an image
    // send a buffer to the camera device
    // wait for camera to fill the buffer
    // read the buffer

    pub fn capture_image(&self) -> Result<bool, String> {
        let start_time = Instant::now();

        // #define VIDIOC_DQBUF		_IOWR('V', 17, struct v4l2_buffer)
        let mut buf: v4l2_buffer = unsafe { std::mem::zeroed() };
        buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
        buf.index = 0;

        let media_fd = self.file.as_raw_fd();

        ioctl_readwrite!(vidioc_dqbuf, VIDIOC_QUERYCAP_MAGIC, 17, v4l2_buffer);
        match unsafe { vidioc_dqbuf(media_fd, &mut buf) } {
            Ok(_) => {
                info!("IOCTL: dqbuf [OK]");
            }
            Err(e) => {
                let elapsed = start_time.elapsed();
                error!("Time at dqbuf failure: {} ms", elapsed.as_millis());
                return Err("dqbuf [FAILED]".to_string());
            }
        }
        // enqueue buffer
        self.enqueue_buffer();

        let elapsed = start_time.elapsed();
        println!("Time after enqueueing buffer: {} ms", elapsed.as_millis());

        Ok(true)
    }
}

use std::time::Instant;
use memmap::MmapMut;

impl Camera { // helper functions
    pub fn enqueue_buffer(&self) -> Result<bool, String> {
        let media_fd = self.file.as_raw_fd();
        // #define VIDIOC_QBUF		_IOWR('V', 15, struct v4l2_buffer)
        let mut buf: v4l2_buffer = unsafe { std::mem::zeroed() };
        buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
        buf.index = 0;

        ioctl_readwrite!(vidioc_qbuf, VIDIOC_QUERYCAP_MAGIC, 15, v4l2_buffer);

        match unsafe { vidioc_qbuf(media_fd, &mut buf) } {
            Ok(_) => {
                info!("IOCTL: qbuf [OK]");
            }
            Err(e) => {
                return Err("qbuf [FAILED]".to_string());
            }
        }
        Ok(true)
    }

}

impl Camera {
    pub fn get_buffer_ptr(&self) -> *const u8 {
        self.buffer.buffer as *const u8
    }

    pub fn get_buffer_length(&self) -> usize {
        self.buffer.length
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        // dequeue buffer
        self.stop_capture();
        // unmap buffer
        unsafe {
            libc::munmap(self.buffer.buffer, self.buffer.length);
        }

    }
}