use std::fs;
use std::io::Write;
use tokio::net::TcpListener;
use tokio::prelude::*;
mod camera;
mod livestream;
mod encode;
mod inference_engine;
mod timer;
mod realtime;
mod api;
#[cfg(windows)]
use camera::camera_win::WindowsCamera;
#[cfg(any(unix, macos))]
use camera::camera_pi::PiCamera;
use crate::camera::CameraProvider;
use crate::livestream::*;
use tungstenite::protocol::Message;
use std::io::Cursor;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use tokio_tungstenite::accept_async as accept_ws;
use crate::encode::encode_outcoming_message;
use chrono::prelude::*;
use tokio::sync::mpsc as achan;
/*
fn main(){
    start_smartpi();

}
#[no_mangle]
pub extern "C" fn start_smartpi(){
    let runtime=tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        tokio_main().await.unwrap();
    });
}
*/


async fn tokio_main(tx: achan::Sender<IncomingMessage>) -> Result<(), Box<dyn std::error::Error>> {
    //#[cfg(windows)]
    //let camera=WindowsCamera::new(640, 480, 60);
    //#[cfg(any(unix, macos))]
    //let camera=PiCamera::new(640, 480, 50);
    //let worker=LiveStream::new(Box::new(camera));
    //let ls=worker.start();

    let mut listener = TcpListener::bind("0.0.0.0:17000").await?;
    loop {
        let sender=tx.clone();
        let (socket, _) = listener.accept().await?;
        //let tx=worker.get_sender();
        tokio::spawn(async move {
            let ws=accept_ws(socket).await.unwrap();
            let mut client=LiveStreamClient::connect(sender).await;
            let info=client.stream_info().await;
            let (mut tx, mut rx)=ws.split();
            //println!("({},{})", info.current_range.0, info.current_range.1);
            let ret=tx.send(Message::binary(encode::encode_outcoming_message(OutcomingMessage::CurrentInfo(info)))).await;
            if let Ok(_)=ret
            {
                while let Some(item) = rx.next().await {
                    match item {
                        Ok(Message::Binary(s)) => {
                            //println!("message: {:?}", s);
                            if s.len() < 9 {
                                continue;
                            }
                            let mut arr: [u8; 8] = [0; 8];
                            for i in 0..=7 {
                                arr[i] = s[i + 1];
                            }
                            let arg = Cursor::new(arr).read_u64::<LittleEndian>().unwrap();
                            match s[0] {
                                0x0 => { // Info
                                    let info = client.stream_info().await;
                                    let ret=tx.send(Message::binary(encode::encode_outcoming_message(OutcomingMessage::CurrentInfo(info)))).await;
                                    match ret{
                                        Err(_)=>{break;}
                                        Ok(_)=>{}
                                    }
                                }
                                0x1 => { //FrameReq
                                    {
                                        //let timer=timer::Timer::new("uncompressed image");
                                    let frame = client.request_batch(arg as usize).await;
                                    //println!("Sending");


                                        let ret = tx.send(Message::binary(encode::encode_outcoming_message(OutcomingMessage::FrameArrive(frame)))).await;
                                    }
                                    //println!("Sent");
                                    match ret{
                                        Err(_)=>{break;}
                                        Ok(_)=>{}
                                    }

                                }
                                0x2 => {}
                                0x3 => {}
                                _ => {}
                            }
                        }
                        Ok(_) => {}
                        Err(x) => {
                            break;
                        }
                    }
                }
            }
            //let (sink, stream)=ws.split();

            client.destroy().await;
        });
    }
}
/*
fn main_camera() {
    println!("Hello, world!");
    let mut camera = rscam::new("/dev/video0").unwrap();

    camera.start(&rscam::Config {
        interval: (1, 60),      // Try to run at 60 fps.
        resolution: (1280, 720),
        format: b"MJPG",
        ..Default::default()
    }).unwrap();

    for i in 0..1000 {
        let frame = camera.capture().unwrap();
        let mut file = fs::File::create(&format!("frame-{}.jpg", i)).unwrap();
        file.write_all(&frame[..]).unwrap();
    }
}
*/

pub fn time_now()->usize{
    let now = Utc::now();
    let stamp=now.timestamp_millis() as usize;
    stamp
}