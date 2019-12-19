#include "py/obj.h"
#include "py/runtime.h"
#include "py/builtin.h"
#include "stdlib.h"
#include "string.h"
extern void* StartPythonCamera(uint64_t width,uint64_t height, uint64_t frame_rate);
extern void* StartPythonInferer(void* livestream, const char* network_path, const char* weight_path);
extern void* StartPythonLivestream();
extern void StartPythonWebsocket(void* livestream, void* camera, void* inferer);

//extern void start_smartpi();
STATIC mp_obj_t startup_livestream(){
    void* livestream=StartPythonLivestream();
    return mp_obj_new_int((uint64_t)livestream);
}
STATIC mp_obj_t startup_inferer(mp_obj_t livestream, mp_obj_t network_path, mp_obj_t weight_path){
    void* stream=(void*)mp_obj_get_int(livestream);
    size_t l;
    const char* path1=mp_obj_str_get_data(network_path, &l);
    char* patha=calloc(l+1, 1);
    memcpy((void*)patha, (void*)path1, l);
    const char* path2=mp_obj_str_get_data(weight_path, &l);
    char* pathb=calloc(l+1, 1);
    memcpy((void*)pathb, (void*)path2, l);
    void* inferer=StartPythonInferer(stream, patha, pathb);
    free(patha);
    free(pathb);
    return mp_obj_new_int((uint64_t)inferer);
}


STATIC mp_obj_t startup_camera(mp_obj_t width, mp_obj_t height, mp_obj_t frame_rate){
    uint64_t w=mp_obj_get_int(width);
    uint64_t h=mp_obj_get_int(height);
    uint64_t fr=mp_obj_get_int(frame_rate);
    void* camera=StartPythonCamera(w, h, fr);
    return mp_obj_new_int((uint64_t)camera);

}

STATIC mp_obj_t startup_websocket(mp_obj_t livestream, mp_obj_t camera, mp_obj_t inferer){
    void* arg1=(void*)mp_obj_get_int(livestream);
    void* arg2=(void*)mp_obj_get_int(camera);
    void* arg3=(void*)mp_obj_get_int(inferer);
    StartPythonWebsocket(arg1, arg2, arg3);
    return mp_obj_new_int(0);

}


STATIC MP_DEFINE_CONST_FUN_OBJ_0(startup_livestream_obj, startup_livestream);
STATIC MP_DEFINE_CONST_FUN_OBJ_3(startup_inferer_obj, startup_inferer);
STATIC MP_DEFINE_CONST_FUN_OBJ_3(startup_camera_obj, startup_camera);
STATIC MP_DEFINE_CONST_FUN_OBJ_3(startup_websocket_obj, startup_websocket);

STATIC const mp_rom_map_elem_t smartpi_global_table[]={
    {MP_ROM_QSTR(MP_QSTR___name__), MP_ROM_QSTR(MP_QSTR_smartpi)},
    {MP_ROM_QSTR(MP_QSTR_startup_livestream), MP_ROM_PTR(&startup_livestream_obj)},
    {MP_ROM_QSTR(MP_QSTR_startup_inferer), MP_ROM_PTR(&startup_inferer_obj)},
    {MP_ROM_QSTR(MP_QSTR_startup_camera), MP_ROM_PTR(&startup_camera_obj)},
    {MP_ROM_QSTR(MP_QSTR_startup_websocket), MP_ROM_PTR(&startup_websocket_obj)},
};

STATIC MP_DEFINE_CONST_DICT(smartpi_globals, smartpi_global_table);


const mp_obj_module_t smartpi_cmodule = {
    .base = {&mp_type_module},
    .globals = (mp_obj_dict_t*)&smartpi_globals,
};

MP_REGISTER_MODULE(MP_QSTR_smartpi, smartpi_cmodule, MODULE_SMARTPI_ENABLED);
