#[cfg(windows)]
pub mod camera_win;
#[cfg(any(unix, macos))]
pub mod camera_pi;
use std::option::Option;
use std::vec::Vec;
use std::sync::Arc;

pub trait CameraProvider: Send+Sync{
    //fn new(width: usize, height: usize, frame_rate: usize)->Self;
    fn capture(&mut self)->Option<Vec<u8>>;
    fn h264_header(&self)->Arc<Vec<u8>>;
}
