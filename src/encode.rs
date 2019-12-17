use tungstenite::protocol::Message;
use std::io::Cursor;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use bytes::buf::BufMut;
use crate::OutcomingMessage;

pub fn encode_outcoming_message(message: OutcomingMessage)->Vec<u8>{

    match message {
        OutcomingMessage::CurrentInfo(info)=>{
            let mut ret=Vec::new();
            ret.reserve(1+8+8+info.h264_header.len());
            ret.put_u64_le(info.current_range.0 as u64);
            ret.put_u64_le(info.current_range.1 as u64);
            ret.put_slice(&info.h264_header);
            ret.put_u8(0);
            ret

        }
        OutcomingMessage::FrameArrive(frame)=>{
            let mut ret=Vec::new();
            match frame{
                Ok(batch)=>{
                    //println!("Encode start");
                    let mut total_size: usize=1+320;
                    total_size+=batch.data.len();

                    ret.reserve(total_size);
                    for f in 0..10{
                        ret.put_u64_le(batch.sizes[f] as u64);
                        ret.put_u64_le(batch.capture_timestamps[f] as u64);
                        ret.put_u64_le(batch.infer_timestamps[f] as u64);
                        ret.put_u64_le(batch.infer_results[f] as u64);
                        //ret.put_slice(f.0.as_slice());
                    }
                    let mut last_size=ret.len();
                    /*
                    for f in 0..10{
                        image::jpeg::JPEGEncoder::new(&mut ret).encode(&batch.0[f*640*480*3..(f+1)*640*480*3], 640, 480, image::ColorType::RGB(8)).unwrap();
                        let new_size=ret.len();
                        unsafe {std::ptr::write(ret.as_mut_ptr().offset((f as isize) *32) as *mut usize, new_size-last_size)};
                        last_size=new_size;
                    }
                    */
                    ret.put_slice(batch.data.as_slice());
                    /*
                    for f in batch.0.iter(){
                        //ret.put_u64_le(f.0.len() as u64);
                        //ret.put_u64_le(f.1 as u64);
                        ret.put_slice(f.as_slice());
                    }
                    */
                    ret.put_u8(1);
                    //println!("Encode end");
                }
                Err(range)=>{
                    ret.reserve(1+8+8);
                    ret.put_u64_le(range.0 as u64);
                    ret.put_u64_le(range.1 as u64);
                    ret.put_u8(2);
                }
            }
            ret
        }
        _=>unreachable!()
    }

}