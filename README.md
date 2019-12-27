SmartPi Software Stack
========

Software stack for SmartPi, embedded video streaming & neural network classification application based on Raspberry Pi 3/3+.

Can run at 60fps, 640x480 resolution.

- [x] Aarch64 Raspberry-Pi Kernel.
- [x] Movidius driver for aarch64-linux-musl and static linking.
- [x] V4L2-based video streaming.
- [x] Train and run NN on Myriad.
- [x] Better replacement for CPython: MicroPython.

Building
========

We have wrapped the world together. So you don't need to fight everything one by one.

Preparations
--------

- aarch64-linux-musl toolchain.
- cargo aarch64-unknown-linux-musl target.
- Properly configured standalone aarch64 linux kernel, with initramfs boot script that executes /boot/ext/init automatically.
- Properly exported inference graph at `sysroot/inference_graph.xml` and `sysroot/inference_graph.bin`.
- For from-scratch users: prepare every static library on your own. Most libraries are from static dldt, but there are also some (e.g. libudev, libusb and libturbojpeg) that needs to be compiled by yourself.

Build
--------

```bash
export CROSS_COMPILE=aarch64-linux-musl-
make
make smartpi.img # For a dd-ready image.

```

Known Bottlenecks and known issues
========

JPEG decompression.
--------

**This is the current performance bottleneck of SmartPi, limiting SmartPi at 60 fps (instead of 90, which is theoretical limit bounded by the camera).**

State of the art implementation uses CPU to do the decoding: to accelerate this, we have to use libturbojpeg and libepeg to generate thumbnail as fast as possible.
However, the latency is about 160ms unpipelined, limiting framerate to no more than 60 fps and breaking compatibility with RT-Preempt, since default preemptive scheduling increases latency.

One way to do this (on **aarch64**, since **userland mmal and OpenMAX** are unavailable on aarch64) is to exploit the v4l2 decoder. However this is not done in current implementation.


Neural network
--------

```
It is a terrible idea to overfit on 64 images and test on other 64 images.
```

Currently SmartPi uses a standard MobileNet, with a fully-connected level to output. This behaves terrible on current test set (60% correctness).

One way is to use a two-step network: one step for cropping (object detection), and the second step for recognizing the cropped image.

Reference: https://github.com/opencv/open_model_zoo/tree/2018/demos/crossroad_camera_demo

Technical details and magics
========

Integrating with MicroPython
--------

```bash
# Please change library paths to your own.
cd ports/unix
export CC=aarch64-linux-musl-gcc
make deplibs
LD=aarch64-linux-musl-g++ \
CFLAGS_MOD="-static" \
LDFLAGS_MOD="-static \
-L /home/gjz010/playground/micropython-1.11/ports/unix/build/lib/libffi/out/lib/  \
-L /mnt/c/links/embed/smartpi/library \
-L /mnt/c/links/embed/smartpi/target/aarch64-unknown-linux-musl/release/ \
-Wl,-Bstatic -Wl,--start-group \
-lade -lmyriadPlugin -linference_engine_s -lusb -ludev -lngraph -lpugixml -lstb_image \
-lvpu_common_lib -lvpu_graph_transformer -lmvnc -lfluid -lXLink -lgflags_nothreads -lstdc++ -lc -lgcc -ljpeg -lturbojpeg \
-lepeg -lexif -ljpeg_sample -linfer_service -lsmartpi -Wl,--end-group" \
C_INCLUDE_PATH=/home/gjz010/playground/micropython-1.11/ports/unix/build/lib/libffi/out/lib/libffi-3.99999/include \
make USER_C_MODULES=../../../modules CFLAGS_EXTRA=-DMODULE_SMARTPI_ENABLED=1
```


Magic spell for compiling statically-linked SmartPI (Debug purpose).
--------

Every time we make the application more static, we benifit more from it.
Change this to executable before you use this.

```bash
# Use --start-group to ignore order. This is necessary since libmyriadPlugin and libinference_engine_s references each other.
# Use libusb, libudev(eudev if you hate systemd), libturbojpeg, libepeg and dldt stuff.
# You need to cross-compile these libraries by themselves.
# Static openvino: https://github.com/gjz010/dldt

RUSTFLAGS="-C linker=aarch64-linux-musl-g++" \
cargo rustc --target aarch64-unknown-linux-musl --release -- \
-C link-args="-L /home/gjz010/playground/dldt/inference-engine/bin/aarch64/MinSizeRel/lib/  \
-L /home/gjz010/playground/libusb/usr/local/lib -L /home/gjz010/eudev/usr/lib -L /home/gjz010/playground/dldt/inference-engine/build/lib \
-Wl,-Bstatic -Wl,--start-group \
-lusb -ludev \
-lade -lmyriadPlugin -linference_engine_s -lngraph -lpugixml -lstb_image -lvpu_common_lib -lvpu_graph_transformer -lmvnc -lfluid -lXLink -lgflags_nothreads \
-lstdc++ -lc -lgcc  \
-ljpeg -lturbojpeg -lepeg -lexif \
-Wl,--end-group \
/opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/9.1.0/crtbeginT.o /opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/9.1.0/crtend.o"
```