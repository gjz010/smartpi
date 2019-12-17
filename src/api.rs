use std::boxed::Box;
struct PythonCamera{

}

struct PythonInferer{


}

struct PythonWebsocket{

}

#[no_mangle]
pub extern "C" fn StartPythonCamera()->usize{


}

#[no_mangle]
pub extern "C" fn StartPythonInferer(camera: usize)->usize{


}

#[no_mangle]
pub extern "C" fn StartPythonWebsocket(inferer: usize)->usize{


}