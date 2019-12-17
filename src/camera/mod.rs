#[cfg(windows)]
pub mod camera_win;
#[cfg(any(unix, macos))]
pub mod camera_pi;
use std::option::Option;
use std::vec::Vec;
use std::sync::Arc;
use bytes::buf::BufMut;

pub trait CameraProvider: Send+Sync{
    //fn new(width: usize, height: usize, frame_rate: usize)->Self;
    fn capture(&mut self)->Option<Vec<u8>>{
        let mut frame=Vec::new();
        self.capture_zerocopy(&mut frame)?;
        Some(frame)
    }
    fn capture_zerocopy(&mut self, target: &mut Vec<u8>)->Option<()>{
        let frame=self.capture()?;
        target.put(&frame[..]);
        Some(())
    }
    fn h264_header(&self)->Arc<Vec<u8>>;
}
