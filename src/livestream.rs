use std::collections::{HashMap, BTreeMap};
use tokio::sync::mpsc::*;

use std::{thread, fs};
use crate::camera::CameraProvider;
use std::sync::Arc;
use std::cell::RefCell;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::error::ErrorKind;
use std::io::Write;
use std::sync::mpsc as bchan;
pub type VideoFrame=(Vec<u8>, usize, usize, usize);
use crate::inference_engine::{start_inference_service, InfererHandler};
use crate::time_now;
// 10 Frames as a batch.
pub struct VideoBatchContent{
    pub data: Vec<u8>,
    pub sizes: [usize; 10],
    pub capture_timestamps: [usize; 10],
    pub infer_timestamps: [usize; 10],
    pub infer_results: [usize; 10]
}
pub struct MutableVideoBatchContent{
    pub data: Vec<u8>,
    pub sizes: [usize; 10],
    pub capture_timestamps: [usize; 10]
}
pub type VideoBatch=Arc<VideoBatchContent>;
pub type MutableVideoBatch=Box<MutableVideoBatchContent>;
pub enum IncomingMessage{
    CameraShot(VideoBatch),
    FrameReq(usize, usize),
    ClientJoin(Sender<OutcomingMessage>),
    ClientQuit(usize),
    QueryInfo(usize)
}
pub struct StreamInfo{
    pub current_range: (usize, usize),
    pub h264_header: Arc<Vec<u8>>
}
pub enum OutcomingMessage{
    FrameArrive(Result<VideoBatch, (usize, usize)>),
    ClientID(usize),
    CurrentInfo(StreamInfo)
}
// A simple single-threaded ring buffer.
pub struct RingBuffer<T: Clone>{
    data: Vec<Option<T>>,
    size: usize,
    start: usize,
    end: usize,
    offset: usize, // real index of offset
    next_index: usize // real index of end
}

impl<T:Clone>  RingBuffer<T>{
    pub fn new(size: usize)->RingBuffer<T>{
        assert!(size>1);
        let mut v=Vec::new();
        for i in 0..size{
            v.push(None);
        }
        RingBuffer{
            data: v,
            size,
            start: 0,
            end: 0,
            offset: 0,
            next_index: 0
        }
    }
    pub fn info(&self){
        println!("<RingBuffer size={}, start={}, end={}, offset={}, next_index={}>", self.size, self.start, self.end, self.offset, self.next_index);
    }
    pub fn fetch(&mut self, index: usize)->Option<T>{
        //println!("fetching frame {} from [{}, {})", index, self.offset, self.next_index);
        if index<self.offset || index>=self.next_index{
            return None;
        }
        let mut idx=index-self.offset+self.start;
        if idx>=self.size{
            idx-=self.size;
        }
        Some(self.data[idx].as_ref().unwrap().clone())
    }
    pub fn push(&mut self, value: T){
        let index=self.next_index;
        self.next_index=index+1;
        self.data[self.end]=Some(value);
        self.end+=1;
        if self.end>=self.size{
            self.end-=self.size;
        }
        if self.end==self.start{ // The ring-buffer is full. Push start ahead.
            self.start+=1;
            if self.start>=self.size{
                self.start-=self.size;
            }
            self.offset+=1;
        }
    }
    pub fn current_range(&self)->(usize, usize){
        (self.offset, self.next_index)
    }
    pub fn fetch_with_err(&mut self, index: usize)->Result<T, (usize, usize)>{
        match self.fetch(index){
            Some(x)=>Ok(x),
            None=>Err(self.current_range())
        }
    }
}


pub struct LiveStream{
    next_client_id: usize,
    clients: BTreeMap<usize, Sender<OutcomingMessage>>,
    cached_frames: RingBuffer<VideoBatch>,
    channel: (Sender<IncomingMessage>, Receiver<IncomingMessage>),
    first_frame: Option<Arc<Vec<u8>>>
}

impl LiveStream{
    pub fn new()->Self{
        LiveStream{
            next_client_id: 0,
            clients: BTreeMap::new(),
            cached_frames: RingBuffer::new(20),
            channel: channel(5),
            first_frame: None
        }
    }
    pub fn get_sender(&self)->Sender<IncomingMessage>{
        self.channel.0.clone()
    }
    pub fn start(mut self, mut camera: Box<CameraProvider>, mut inferer: Box<InfererHandler>, runtime: &mut tokio::runtime::Runtime)->Sender<IncomingMessage>{
        let mut sender=self.get_sender();
        let ret=sender.clone();
        println!("Taking first frame");
        //let mut camera=camera.take().unwrap();
        self.first_frame=Some(camera.h264_header());
        //let mut inferer=inferer.take().unwrap();
        // Start camera thread.
        std::thread::spawn(move ||{
            let mut i:usize=0;
            use std::time::Instant;
            let mut now = Instant::now();
            loop {


                //println!("camera {}", i);
                i=i+1;
                let msg=Box::new({
                    let mut buffer=Vec::new();
                    buffer.reserve(640*480*3*10);
                    let mut timestamps=[0 as usize; 10];
                    let mut old_size=0;
                    let mut sizes=[0; 10];
                    for i in 0..=9{
                        camera.capture_zerocopy(&mut buffer).unwrap();
                        timestamps[i]=time_now();
                        sizes[i]=buffer.len()-old_size;
                        old_size=buffer.len();
                    }
                    MutableVideoBatchContent{data: buffer, sizes, capture_timestamps: timestamps}

                });
                /*
                let mut msg= ({
                    let mut data: [std::mem::MaybeUninit<Option<(Vec<u8>, usize)>>; 10] = unsafe {
                        std::mem::MaybeUninit::uninit().assume_init()
                    };

                    for elem in &mut data[..] {
                        unsafe { std::ptr::write(elem.as_mut_ptr(), Some({
                            let pic=camera.capture().unwrap();
                            let stamp=time_now();
                            (pic, stamp)

                        })); }
                    }
                    let batch=unsafe { std::mem::transmute::<_, [Option<(Vec<u8>, usize)>; 10]>(data) };
                    //let mut file = fs::File::create(&format!("frame-{}.264", i)).unwrap();
                    //for i in batch.iter(){
                    //    file.write_all(&i.0).unwrap();
                    //}

                    batch
                });
                */
                //println!("sending to inferer");
                inferer.send(msg).unwrap();
                //println!("sent");
                /*
                loop {
                    let ret=sender.try_send(msg);
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
                */
                if i%2==0{
                    let elapsed = now.elapsed();
                    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
                    println!("i={} sec={} FPS={}", i*10, sec, 20.0/sec);
                    now = Instant::now();

                }
                //std::thread::sleep(std::time::Duration::new(1, 0));
            }

        });
        // Start tokio coroutine
        runtime.spawn (async move {
            loop{
                let msg=self.channel.1.recv().await.unwrap();
                self.handle_message(msg).await;
            }

        });
        return ret;
    }


    pub async fn handle_message(&mut self, msg: IncomingMessage){
        match msg{
            IncomingMessage::CameraShot(video)=>{
                self.cached_frames.push(video);
                //self.cached_frames.info();
            }
            IncomingMessage::FrameReq(client_id, frame_id)=>{
                let sender=self.clients.get(&client_id).unwrap();
                sender.clone().send(OutcomingMessage::FrameArrive(self.cached_frames.fetch_with_err(frame_id))).await.ok().unwrap();
            }
            IncomingMessage::ClientJoin(sender)=>{
                let id=self.next_client_id;
                self.next_client_id+=1;
                sender.clone().send(OutcomingMessage::ClientID(id)).await.ok().unwrap();
                self.clients.insert(id, sender.clone());
            }
            IncomingMessage::ClientQuit(client_id)=>{
                self.clients.remove(&client_id);
            }
            IncomingMessage::QueryInfo(client_id)=>{
                let sender=self.clients.get(&client_id).unwrap();
                sender.clone().send(OutcomingMessage::CurrentInfo(StreamInfo{
                    current_range: self.cached_frames.current_range(),
                    h264_header: Arc::clone(&self.first_frame.as_ref().unwrap())
                })).await.ok().unwrap();
            }
        }
    }

}

pub struct LiveStreamClient{
    index: usize,
    stream: Sender<IncomingMessage>,
    receiver: Receiver<OutcomingMessage>
}

impl LiveStreamClient{
    pub async fn connect(stream: Sender<IncomingMessage>)->LiveStreamClient{
        let (tx, mut rx)=channel(5);
        stream.clone().send(IncomingMessage::ClientJoin(tx)).await.ok().unwrap();
        match rx.recv().await.unwrap() {
            OutcomingMessage::ClientID(index)=>{
                LiveStreamClient{
                    index,
                    stream,
                    receiver: rx
                }
            }
            _=>unreachable!()
        }

    }
    pub async fn stream_info(&mut self)->StreamInfo{
        self.stream.clone().send(IncomingMessage::QueryInfo(self.index)).await.ok().unwrap();
        match self.receiver.recv().await.unwrap(){
            OutcomingMessage::CurrentInfo(info)=>{
                info
            }
            _=>unreachable!()
        }
    }

    pub async fn request_batch(&mut self, index: usize)->Result<VideoBatch, (usize, usize)>{
        self.stream.clone().send(IncomingMessage::FrameReq(self.index, index)).await.ok().unwrap();
        match self.receiver.recv().await.unwrap(){
            OutcomingMessage::FrameArrive(info)=>{
                info
            }
            _=>unreachable!()
        }
    }
    pub async fn destroy(&mut self){
        self.stream.clone().send(IncomingMessage::ClientQuit(self.index)).await.ok().unwrap();
    }
}