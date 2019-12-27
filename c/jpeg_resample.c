#include "Epeg.h"
#include "stdint.h"
#include "string.h"
void JPEGResample(uint8_t* data, int32_t size, int32_t w, int32_t h, uint8_t* dst){
        Epeg_Image* img;
        img=epeg_memory_open(data, size);
        epeg_decode_size_set(img, w, h);
        epeg_decode_colorspace_set(img, EPEG_RGB8);
        const void* thumb=epeg_pixels_get(img, 0, 0, w, h);
        memcpy(dst, thumb, w*h*3);
        epeg_pixels_free(img, thumb);
        epeg_close(img);
}
