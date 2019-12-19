use std::boxed::Box;
use crate::camera::CameraProvider;
use crate::inference_engine::{InfererHandler, start_inference_service};
use crate::camera::camera_pi::PiCamera;
use crate::livestream::LiveStream;
use crate::tokio_main;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::thread::spawn;

struct PythonCamera{
    camera: Box<CameraProvider>
}

struct PythonInferer{
    inferer: Box<InfererHandler>

}
struct PythonLiveStream{
    livestream: Box<LiveStream>
}

#[no_mangle]
pub extern "C" fn StartPythonCamera(width: usize, height: usize, frame_rate: usize)->usize{
    println!("[SmartPi] Starting camera");
    let camera=PiCamera::new(width, height, frame_rate);
    println!("[SmartPi] Camera started");
    Box::leak(Box::new(PythonCamera{camera: Box::new(camera)}) )as *mut _ as usize
}

#[no_mangle]
pub extern "C" fn StartPythonInferer(ptrlivestream: usize, network_path: *const c_char, weight_path: *const c_char)->usize{
    let livestream=unsafe {&*(ptrlivestream as *const PythonLiveStream)};
    println!("[SmartPi] Starting inferer");
    let patha=unsafe {CStr::from_ptr(network_path).to_str().unwrap()};
    let pathb=unsafe {CStr::from_ptr(weight_path).to_str().unwrap()};
    let sender=livestream.livestream.get_sender();
    let mut inferer=start_inference_service(patha, pathb, sender);
    println!("[SmartPi] Inferer started");
    Box::leak(Box::new(PythonInferer{inferer: Box::new(inferer)})) as *mut _ as usize


}

#[no_mangle]
pub extern "C" fn StartPythonLivestream()->usize{
    println!("[SmartPi] Starting livestream");
    let mut livestream=LiveStream::new();
    println!("[SmartPi] Livestream started");
    Box::leak(Box::new(PythonLiveStream{livestream: Box::new(livestream)})) as *mut _ as usize
}

#[no_mangle]
pub extern "C" fn StartPythonWebsocket(ptrlivestream: usize, camera: usize, inferer: usize){
    let livestream_obj=unsafe {Box::from_raw(ptrlivestream as *mut PythonLiveStream)};
    let camera_obj=unsafe {Box::from_raw(camera as *mut PythonCamera)};
    let inferer_obj=unsafe {Box::from_raw(inferer as *mut PythonInferer)};
    println!("[SmartPi] Starting livestream websocket");
    let PythonLiveStream{livestream}=*livestream_obj;
    let PythonCamera{camera}=*camera_obj;
    let PythonInferer{inferer}=*inferer_obj;
    let mut runtime=tokio::runtime::Runtime::new().unwrap();
    let sender= livestream.start(camera, inferer, &mut runtime);
    spawn(move || {

        runtime.block_on(async {
            tokio_main(sender).await.unwrap();
        });
    });

    println!("[SmartPi] Livestream websocket started.");

}