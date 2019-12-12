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
                    let mut total_size: usize=1;
                    for f in batch.iter(){
                        total_size+=f.0.len();
                        total_size+=16;
                    }
                    ret.reserve(total_size);
                    for f in batch.iter(){
                        ret.put_u64_le(f.0.len() as u64);
                        ret.put_u64_le(f.1 as u64);
                        //ret.put_slice(f.0.as_slice());
                    }
                    for f in batch.iter(){
                        //ret.put_u64_le(f.0.len() as u64);
                        //ret.put_u64_le(f.1 as u64);
                        ret.put_slice(f.0.as_slice());
                    }
                    ret.put_u8(1);
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