use opencv::{
	prelude::*,
	imgproc::*,
	core::*,
};
use rayon::prelude::*;
use crate::types::{Image, COLOR_SPACE};

pub fn resize_with_padding(img: &Mat, new_shape: [i32;2]) -> Mat {
	let img_shape = [img.cols(), img.rows()];
	let width: i32;
	let height: i32;
	if img_shape[0] as f64 / img_shape[1] as f64 > new_shape[0] as f64 / new_shape[1] as f64 {
		width = new_shape[0];
		height = (new_shape[0] as f64 / img_shape[0] as f64 * img_shape[1] as f64) as i32;
	} else {
		width = (new_shape[1] as f64 / img_shape[1] as f64 * img_shape[0] as f64) as i32;
		height = new_shape[1];
	}

	let mut resized = Mat::default();
	resize(
		img,
		&mut resized,
		Size { width, height },
		0.0, 0.0,
		INTER_LINEAR)
		.expect("resize_with_padding: resize [FAILED]");

	let delta_w = new_shape[0] - width;
	let delta_h = new_shape[1] - height;
	let (top, bottom) = (delta_h / 2, delta_h - delta_h / 2);
	let (left, right) = (delta_w / 2, delta_w - delta_w / 2);
		
	let mut rslt = Mat::default();
	copy_make_border(
		&resized,
		&mut rslt,
		top, bottom, left, right,
		BORDER_CONSTANT,
		Scalar::new(0.0, 0.0, 0.0, 0.0))
		.expect("resize_with_padding: copy_make_border [FAILED]");
	rslt
}

pub fn draw_keypoints(img: &mut Mat, keypoints: &[f32], threshold: f32) {
	// keypoints: [1, 17, 3]
	let base: f32;
	let pad_x: i32;
	let pad_y: i32;
	if img.rows() > img.cols() {
		base = img.rows() as f32;
		pad_x = (img.rows() - img.cols()) / 2;
		pad_y = 0;
	} else {
		base = img.cols() as f32;
		pad_x = 0;
		pad_y = (img.cols() - img.rows()) / 2;
	}

	for index in 0..17 {
		let y_ratio = keypoints[index * 3];
		let x_ratio = keypoints[index * 3 + 1];
		let confidence = keypoints[index * 3 + 2];
		if confidence > threshold {
			circle(img,
				Point { x: (x_ratio * base) as i32 - pad_x, y: (y_ratio * base) as i32 - pad_y},
				0,
				Scalar::new(0.0, 255.0, 0.0, 0.0),
				5, LINE_AA, 0).expect("Draw circle [FAILED]");
		}
	}
}


// pub fn resize_with_padding_no_opencv(
// 	img: &Image,
// 	(new_width, new_height): (i32, i32)
// ) -> Image {
// 	let (original_width, original_height) = (img.width, img.height);
// 	// Calculate scaling ratios
// 	let width_ratio = if original_width as f64 / original_height as f64 > new_width as f64 / new_height as f64 {
// 		new_width as f64 / original_width as f64
// 	} else {
// 		new_height as f64 / original_height as f64
// 	};
//
// 	let scaled_width = (original_width as f64 * width_ratio) as i32;
// 	let scaled_height = (original_height as f64 * width_ratio) as i32;
//
// 	// First resize the image
// 	let resized = resize_image_bilinear(
// 		img, (scaled_width, scaled_height)
// 	);
//
// 	// Calculate padding
// 	let delta_w = new_width - scaled_width;
// 	let delta_h = new_height - scaled_height;
// 	let pad_left = delta_w / 2;
// 	let pad_top = delta_h / 2;
//
// 	// Create final image with padding
// 	let mut final_image = vec![0u8; (new_width * new_height * 3) as usize];
//
// 	// Parallel copy of resized image to padded image
// 	final_image.par_chunks_exact_mut(new_width as usize * 3)
// 		.enumerate()
// 		.for_each(|(y, row)| {
// 			if y >= pad_top as usize && y < (pad_top + scaled_height) as usize {
// 				let src_y = y - pad_top as usize;
// 				let src_row_start = src_y * scaled_width as usize * 3;
// 				let dst_start = (pad_left * 3) as usize;
// 				let dst_end = dst_start + (scaled_width * 3) as usize;
// 				row[dst_start..dst_end].copy_from_slice(
// 					&resized[src_row_start..src_row_start + (scaled_width * 3) as usize]
// 				);
// 			}
// 		});
//
// 	Image{
// 		timestamp: img.timestamp,
// 		width: new_width,
// 		height: new_height,
// 		data: final_image,
//
// 	}
// }
//
// fn resize_image_bilinear(
// 	src_image: &Image,
// 	(dst_width, dst_height): (i32, i32)
// ) -> Vec<u8> {
// 	let src = &src_image.data;
// 	let (src_width, src_height) = (src_image.width, src_image.height);
// 	let mut dst = vec![0u8; (dst_width * dst_height * 3) as usize];
// 	let x_ratio = src_width as f64 / dst_width as f64;
// 	let y_ratio = src_height as f64 / dst_height as f64;
//
// 	dst.par_chunks_exact_mut(dst_width as usize * 3)
// 		.enumerate()
// 		.for_each(|(i, row)| {
// 			let y = i as f64 * y_ratio;
// 			let y1 = y.floor() as i32;
// 			let y2 = (y1 + 1).min(src_height - 1);
// 			let y_diff = y - y1 as f64;
//
// 			for j in 0..dst_width {
// 				let x = j as f64 * x_ratio;
// 				let x1 = x.floor() as i32;
// 				let x2 = (x1 + 1).min(src_width - 1);
// 				let x_diff = x - x1 as f64;
//
// 				let get_src_pixel = |x: i32, y: i32, c: usize| -> f64 {
// 					src[((y * src_width + x) * 3 + c as i32) as usize] as f64
// 				};
//
// 				for c in 0..3 {
// 					let q11 = get_src_pixel(x1, y1, c);
// 					let q21 = get_src_pixel(x2, y1, c);
// 					let q12 = get_src_pixel(x1, y2, c);
// 					let q22 = get_src_pixel(x2, y2, c);
//
// 					let pixel = (q11 * (1.0 - x_diff) * (1.0 - y_diff) +
// 						q21 * x_diff * (1.0 - y_diff) +
// 						q12 * (1.0 - x_diff) * y_diff +
// 						q22 * x_diff * y_diff) as u8;
// 					row[(j * 3) as usize + c] = pixel;
// 				}
// 			}
// 		});
//
// 	dst
// }
pub fn resize_with_padding_ultra_fast(
	img: &Image,
	(new_width, new_height): (i32, i32),
	color_type: COLOR_SPACE
) -> Image {
	let (original_width, original_height) = (img.width, img.height);
	let channels = match(color_type) {
		COLOR_SPACE::RGB => 3,
		COLOR_SPACE::YUV => 2,
		_ => panic!("Invalid color type"),
	};

	// Calculate scaling to maintain aspect ratio
	let scale = if (original_width * new_height) > (original_height * new_width) {
		new_width as f32 / original_width as f32
	} else {
		new_height as f32 / original_height as f32
	};

	let scaled_width = (original_width as f32 * scale) as i32;
	let scaled_height = (original_height as f32 * scale) as i32;

	// First resize the image
	let resized = resize_fast_downsample(img, (scaled_width, scaled_height), channels);

	// Calculate padding
	let delta_w = new_width - scaled_width;
	let delta_h = new_height - scaled_height;
	let pad_left = delta_w / 2;
	let pad_top = delta_h / 2;

	let mut final_image = vec![0u8; (new_width * new_height * channels as i32) as usize];

	// Copy resized image into padded final image
	final_image.par_chunks_exact_mut(new_width as usize * channels)
		.enumerate()
		.for_each(|(y, row)| {
			if y >= pad_top as usize && y < (pad_top + scaled_height) as usize {
				let src_y = y - pad_top as usize;
				let src_row_start = src_y * scaled_width as usize * channels;
				let dst_start = (pad_left * channels as i32) as usize;
				let dst_end = dst_start + (scaled_width * channels as i32) as usize;
				row[dst_start..dst_end].copy_from_slice(
					&resized[src_row_start..src_row_start + (scaled_width * channels as i32) as usize]
				);
			}
		});

	Image {
		timestamp: img.timestamp,
		width: new_width,
		height: new_height,
		data: final_image,
		color_space: color_type,
	}
}




fn resize_fast_downsample(
	src_image: &Image,
	(dst_width, dst_height): (i32, i32),
	channels: usize,
) -> Vec<u8> {
	let src = &src_image.data;
	let (src_width, src_height) = (src_image.width, src_image.height);
	let mut dst = vec![0u8; (dst_width * dst_height * channels as i32) as usize];

	// Calculate step sizes
	let x_step = (src_width as f32 / dst_width as f32).max(1.0) as i32;
	let y_step = (src_height as f32 / dst_height as f32).max(1.0) as i32;

	dst.par_chunks_exact_mut(dst_width as usize * channels)
		.enumerate()
		.for_each(|(y, row)| {
			let src_y = (y as i32 * y_step) as usize;
			for x in 0..dst_width as usize {
				let src_x = (x as i32 * x_step) as usize;
				let src_idx = (src_y * src_width as usize + src_x) * channels;
				let dst_idx = x * channels;

				// Direct copy of pixel values
				for c in 0..channels {
					row[dst_idx + c] = src[src_idx + c];
				}
			}
		});

	dst
}

// https://stackoverflow.com/questions/28079010/rgb-to-ycbcr-using-simd-vectors-lose-some-data
pub fn yuv422_to_rgb24(in_buf: &[u8], out_buf: &mut [u8]) {
	debug_assert_eq!(out_buf.len(), in_buf.len() * 3/2, "Output buffer length must be 3/2 of input buffer length");

	in_buf
		.par_chunks_exact(4)
		.zip(out_buf.par_chunks_exact_mut(6))
		.for_each(|(chunk, out)| {
			let (y1, cb, y2, cr) = (chunk[0], chunk[1], chunk[2], chunk[3]);

			let (r1, g1, b1) = ycbcr_to_rgb((y1, cb, cr));
			let (r2, g2, b2) = ycbcr_to_rgb((y2, cb, cr));

			out[0] = b1;
			out[1] = g1;
			out[2] = r1;
			out[3] = b2;
			out[4] = g2;
			out[5] = r2;
		});
}

pub fn rgb24_to_yuv422(in_buf: &[u8], out_buf: &mut [u8]) {
	debug_assert_eq!(out_buf.len(), in_buf.len() * 2/3, "Output buffer length must be 2/3 of input buffer length");

	in_buf
		.par_chunks_exact(6)
		.zip(out_buf.par_chunks_exact_mut(4))
		.for_each(|(chunk, out)| {
			let (r1, g1, b1) = (chunk[2], chunk[1], chunk[0]);
			let (r2, g2, b2) = (chunk[5], chunk[4], chunk[3]);

			let (y1, cb, cr) = rgb_to_ycbcr((r1, g1, b1));
			let (y2, _, _) = rgb_to_ycbcr((r2, g2, b2));

			out[0] = y1;
			out[1] = cb;
			out[2] = y2;
			out[3] = cr;
		});
}
#[inline]
fn clamp(val: f32) -> u8 {
	val.max(0.0).min(255.0).round() as u8
}

#[inline]
pub fn rgb_to_ycbcr((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
	let r = r as f32;
	let g = g as f32;
	let b = b as f32;

	let y = 0.299 * r + 0.587 * g + 0.114 * b;
	let cb = -0.168736 * r - 0.331264 * g + 0.5 * b + 128.0;
	let cr = 0.5 * r - 0.418688 * g - 0.081312 * b + 128.0;

	(clamp(y), clamp(cb), clamp(cr))
}
#[inline]
pub fn ycbcr_to_rgb((y, cb, cr): (u8, u8, u8)) -> (u8, u8, u8) {
	let y = y as f32;
	let cb = cb as f32 - 128.0;
	let cr = cr as f32 - 128.0;

	let r = y + 1.402 * cr;
	let g = y - 0.344136 * cb - 0.714136 * cr;
	let b = y + 1.772 * cb;

	(clamp(r), clamp(g), clamp(b))
}