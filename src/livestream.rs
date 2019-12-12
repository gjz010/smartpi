use std::collections::{HashMap, BTreeMap};
use tokio::sync::mpsc::*;

use std::{thread, fs};
use crate::camera::CameraProvider;
use std::sync::Arc;
use std::cell::RefCell;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::error::ErrorKind;
use std::io::Write;

type VideoFrame=(Vec<u8>, usize);

// 10 Frames as a batch.
type VideoBatch=[VideoFrame; 10];

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
    camera: Option<Box<CameraProvider>>,
    first_frame: Option<Arc<Vec<u8>>>
}

impl LiveStream{
    pub fn new(camera: impl CameraProvider + 'static)->Self{
        LiveStream{
            next_client_id: 0,
            clients: BTreeMap::new(),
            cached_frames: RingBuffer::new(100),
            channel: channel(5),
            camera: Some(Box::new(camera)),
            first_frame: None
        }
    }
    pub fn get_sender(&self)->Sender<IncomingMessage>{
        self.channel.0.clone()
    }
    pub fn start(mut self)->Sender<IncomingMessage>{
        let mut sender=self.get_sender();
        let ret=sender.clone();
        let mut camera=self.camera.take().unwrap();
        self.first_frame=Some(camera.h264_header());
        // Start camera thread.
        std::thread::spawn(move ||{
            let mut i=0;
            use std::time::Instant;
            let now = Instant::now();
            loop {


                //println!("camera {}", i);
                i=i+1;

                let mut msg= IncomingMessage::CameraShot({
                    let mut data: [std::mem::MaybeUninit<VideoFrame>; 10] = unsafe {
                        std::mem::MaybeUninit::uninit().assume_init()
                    };

                    for elem in &mut data[..] {
                        unsafe { std::ptr::write(elem.as_mut_ptr(), (camera.capture().unwrap(), 1)); }
                    }
                    let batch=unsafe { std::mem::transmute::<_, VideoBatch>(data) };
                    //let mut file = fs::File::create(&format!("frame-{}.264", i)).unwrap();
                    //for i in batch.iter(){
                    //    file.write_all(&i.0).unwrap();
                    //}

                    batch
                });

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

                if i%2==0{
                    let elapsed = now.elapsed();
                    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
                    println!("i={} sec={} FPS={}", i*10, sec, ((i*10) as f64)/sec);

                }
                //std::thread::sleep(std::time::Duration::new(1, 0));
            }

        });
        // Start tokio coroutine
        tokio::spawn (async move {
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