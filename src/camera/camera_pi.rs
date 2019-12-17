#[cfg(any(unix, macos))]
use rscam;
use crate::camera::CameraProvider;
use std::vec::*;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use bytes::buf::BufMut;
pub struct PiCamera {
    width: usize,
    height: usize,
    frame_rate: usize,
    device : rscam::Camera,
    first_frame: Arc<Vec<u8>>
}

unsafe impl Send for PiCamera{}
unsafe impl Sync for PiCamera{}
impl PiCamera{
    pub fn new(width: usize, height: usize, frame_rate: usize) -> Self {
        let w=width as u32;
        let h=height as u32;
        let mut camera = rscam::new("/dev/video0").unwrap();

        camera.start(&rscam::Config {
            interval: (1, frame_rate as u32),      // Try to run at 60 fps.
            resolution: (w, h),
            format: b"MJPG",
            ..Default::default()
        }).unwrap();
        println!("PiCamera initialized.");
        let mut header=Vec::new();
        for i in 1..10{
            let frame=camera.capture().unwrap();
            header.put_slice(&frame[..]);
        }

        PiCamera{width, height, frame_rate, device: camera, first_frame: Arc::new(header)}
    }

}
impl CameraProvider for PiCamera {
    fn capture_zerocopy(&mut self, target: &mut Vec<u8>) -> Option<()> {
        let frame=self.device.capture().unwrap();
        target.put(&frame[..]);
        Some(())

    }
    fn h264_header(&self)->Arc<Vec<u8>>{
        Arc::clone(&self.first_frame)
    }
}