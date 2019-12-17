use escapi;
use super::CameraProvider;
use image;
use std::option::Option;
use image::ImageFormat;
use std::sync::Arc;

pub struct WindowsCamera {
    width: usize,
    height: usize,
    frame_rate: usize,
    device : escapi::Device,
    last_frame : Option<Vec<u8>>,
    counter: usize

}
unsafe impl Send for WindowsCamera{}
unsafe impl Sync for WindowsCamera{}
impl WindowsCamera{
    pub fn new(width: usize, height: usize, frame_rate: usize) -> Self {
        let w=width as u32;
        let h=height as u32;
        let device=escapi::init(0, w, h, frame_rate as u64).unwrap();
        println!("WindowsCamera initialized. device name: {}", device.name());
        WindowsCamera{width, height, frame_rate, device, last_frame: None, counter: 0}
    }
}
impl CameraProvider for WindowsCamera {


    // Simulating 60fps using 30fps.
    fn capture(&mut self) -> Option<Vec<u8>> {
        match &self.last_frame{
            None=>{
                let mut camera=&mut self.device;
                let (width, height) = (camera.capture_width(), camera.capture_height());
                //println!("capture1");
                let pixels = camera.capture().unwrap();

                //println!("capture2");
                let mut buffer = vec![0; width as usize * height as usize * 3];
                for i in 0..pixels.len() / 4 {
                    buffer[i * 3] = pixels[i * 4 + 2];
                    buffer[i * 3 + 1] = pixels[i * 4 + 1];
                    buffer[i * 3 + 2] = pixels[i * 4];
                }
                //println!("capture3");

                let mut buf=Vec::new();

                image::jpeg::JPEGEncoder::new(&mut buf).encode(&buffer, width, height, image::ColorType::RGB(8)).unwrap();
                self.last_frame=Some(buf.clone());
                self.counter=3;
                Some(buf)
            }
            Some(vec)=>{
                let buf=self.last_frame.as_ref().unwrap().clone();
                self.counter-=1;
                if self.counter==0{
                    self.last_frame=None;
                }
                Some(buf)
            }
        }

    }

    fn h264_header(&self) -> Arc<Vec<u8>> {
        Arc::new(Vec::new())
    }
}