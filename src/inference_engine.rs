use std::sync::{mpsc as bchan, Arc};
use tokio::sync::mpsc as achan;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use crate::livestream::{IncomingMessage, MutableVideoBatch, VideoFrame, MutableVideoBatchContent, VideoBatchContent};
use std::thread::spawn;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::error::ErrorKind;
use std::thread;
use image::jpeg::JPEGDecoder;
use bytes::buf::BufMut;
//use semaphore::Semaphore;
struct Core(usize);
struct ExecutableNetwork(usize);
struct InferRequest(usize, Vec<u8>);
use crate::time_now;
#[link(name = "infer_service")]
extern "C" {
    #[no_mangle]
    fn InitializeInferService(network_path: *const c_char, weight_path: *const c_char, use_cpu: usize, core_ret: *mut usize, nn_ret: *mut usize);
    #[no_mangle]
    fn InferBatch(nn: usize, blobdata: *mut u8, size: isize, batch_size: u32, channels: u32, height: u32, width: u32)->usize;
    #[no_mangle]
    fn PollInferResult(preq: usize, ret: *mut f32, size: isize);
}

#[link(name = "jpeg_sample")]
extern "C" {
    #[no_mangle]
    fn JPEGResample(data: *const u8, size: i32, w: i32, h: i32, dst: *mut u8);

}
unsafe fn jpeg_resample(data: &[u8], w: usize, h: usize, dst: &mut [u8]){
    JPEGResample(data.as_ptr(), data.len() as i32, w as i32, h as i32, dst.as_mut_ptr());
}
unsafe fn initialize_inference_engine(network_path: &str, weight_path: &str)->(Core, ExecutableNetwork){
    let use_cpu=0;
    let s1=CString::new(network_path).expect("CString::new() failed");
    let s2=CString::new(weight_path).expect("CString::new() failed");
    let mut core_ret=0;
    let mut nn_ret=0;
    InitializeInferService(s1.as_ptr(), s2.as_ptr(), use_cpu, &mut core_ret as *mut usize, &mut nn_ret as *mut usize);
    (Core(core_ret), ExecutableNetwork(nn_ret))
}

unsafe fn infer_batch(nn: &mut ExecutableNetwork, mut blob: Vec<u8>, batch_size: u32, channels: u32, height: u32, width: u32)->InferRequest{
    let ret=InferBatch(nn.0, blob.as_mut_ptr(), blob.len() as isize, batch_size, channels, height, width);
    InferRequest(ret, blob)

}

unsafe fn poll_infer_result(req: InferRequest, ret: &mut [f32])->Vec<u8>{
    PollInferResult(req.0, ret.as_mut_ptr(), ret.len() as isize);
    req.1
}

fn inference_service_req(mut core: Core, mut nn: ExecutableNetwork, network_path: &str, weight_path: &str, input: bchan::Receiver<MutableVideoBatch>, output: bchan::SyncSender<(Option<Vec<u8>>, [usize; 10],[usize; 10], Option<InferRequest>)>){
    //let mut test_image=image::open(("C:\\links\\embed\\ai\\trainset\\scissor\\1015216984-thumb0969.jpg.jpg")).unwrap();
    //let test_image=test_image.resize_exact(640, 480, image::FilterType::Gaussian);
    //test_image.save("c:\\links\\embed\\hint.png").unwrap();
    for mut msg in input{
        //let mut buffer:Vec<u8>=Vec::new();
        //buffer.reserve(640*480*3*10);
        //for jpeg in msg.iter(){
            //let img=image::load_from_memory_with_format(jpeg.as_ref().unwrap().0.as_ref(), image::ImageFormat::JPEG).unwrap();
            //buffer.put(img.raw_pixels());

        //}
        //println!("len={}", buffer.len());
        //let req=unsafe {infer_batch(&mut nn, buffer, 10, 3, 480, 640)};

        let MutableVideoBatchContent{data, sizes, capture_timestamps}=*msg;
        let mut offset=0;
        let mut blob=Vec::new();
        blob.resize(192*192*3*10, 0);
        let mut index=0;
        for elem in sizes.iter(){
            let jpeg=&data[offset..offset+*elem];
            let dst=&mut blob[index*192*192*3..(index+1)*192*192*3];
            unsafe {jpeg_resample(jpeg, 192, 192, dst);}
            //let img=image::load_from_memory_with_format(jpeg, image::ImageFormat::JPEG).unwrap();
            //img.resize(192, 192, image::Nearest);

            offset+=*elem;
            index=index+1;
        }


        let req=unsafe {infer_batch(&mut nn, blob, 10, 3, 192, 192)};
        output.send((Some(data), sizes, capture_timestamps, Some(req))).unwrap();
    }
}

fn argmax(arr: &[f32], offset: usize)->usize{
    let mut index=0;
    let mut score=arr[offset];
    if arr[offset+1]>score {score=arr[offset+1]; index=1};
    if arr[offset+2]>score {score=arr[offset+2]; index=2};
    if arr[offset+3]>score {score=arr[offset+3]; index=3};
    index
}

fn inference_service_res(input: bchan::Receiver<(Option<Vec<u8>>, [usize; 10], [usize; 10], Option<InferRequest>)>, mut output: achan::Sender<IncomingMessage>){
    let mut ret:[f32;40]=[0.0f32;40];
    for (img, mut sizes, mut timestamps, req) in input{
        unsafe{
            poll_infer_result(req.unwrap(), &mut ret);
            let mut msg= IncomingMessage::CameraShot({
                //let (mut img, mut timestamps)=*img;
                let mut infer_timestamp:[usize; 10]=[time_now(); 10];
                let mut infer_result: [usize; 10]=[0; 10];
                //let mut index=0;
                println!("Req: {} {} {} {} {} {} {} {}", ret[0], ret[1], ret[2], ret[3], ret[4], ret[5], ret[6], ret[7]);
                //let dr=match argmax(&ret, index*4) {0=>"nothing", 1=>"paper", 2=>"scissor", 3=>"rock", _=>"nothing!"};
                //println!("Result = {}", dr);

                for index in 0..10{
                    //let pair=elem.take().unwrap();
                    infer_result[index]=argmax(&ret, index*4);
                    //unsafe { std::ptr::write(data[index].as_mut_ptr(), (pair.0, pair.1, time_now(), argmax(&ret, index*4))); }
                    //index=index+1;
                }

                Arc::new(VideoBatchContent{data: img.unwrap(), sizes, capture_timestamps: timestamps, infer_timestamps: infer_timestamp, infer_results: infer_result})
                //let mut file = fs::File::create(&format!("frame-{}.264", i)).unwrap();
                //for i in batch.iter(){
                //    file.write_all(&i.0).unwrap();
                //}

            });
            loop {
                let ret=output.try_send(msg);
                match ret{
                    Ok(())=>{
                        break;
                    }
                    Err(TrySendError{kind: ErrorKind::NoCapacity, value:p})=>{
                        msg=p;
                    }
                    Err(TrySendError{kind: ErrorKind::Closed, value:p})=>{
                        panic!("Closed!");
                    }
                }
            }
        }
    }
}
pub fn start_inference_service(network_path: &str, weight_path: &str, chan: achan::Sender<IncomingMessage>)->bchan::SyncSender<MutableVideoBatch>{
    let (core, nn)=unsafe {initialize_inference_engine(network_path, weight_path)};
    let (tx, rx)=bchan::sync_channel(8);
    let (itx, irx)=bchan::sync_channel(8);
    let sa=String::from(network_path);
    let sb=String::from(weight_path);
    //let sem=Arc::new(Semaphore::new(10, ()));
    spawn(move || {
        inference_service_req(core, nn, &sa, &sb, rx, itx);
    });
    spawn(move || {
        inference_service_res(irx, chan);
    });
    tx
}