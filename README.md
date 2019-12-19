SmartPi Software Stack
========

Software stack for SmartPi, embedded video streaming & neural network classification application based on Raspberry Pi 3.

Can run at 50fps, 640x480 resolution.

- [x] Aarch64 Raspberry-Pi Kernel
- [x] Movidius driver for aarch64-linux-musl.
- [x] V4L2-based video streaming.
- [x] Train and run NN on Myriad.
- [x] Better replacement for CPython.


Magic spell for compiling statically-linked SmartPI (Debug purpose).
========

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

Integrating with MicroPython
========

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
-lepeg -lexif -ljpeg_sample -linfer_service -lsmartpi -Wl,--end-group"\
C_INCLUDE_PATH=/home/gjz010/playground/micropython-1.11/ports/unix/build/lib/libffi/out/lib/libffi-3.99999/include \
make USER_C_MODULES=../../../modules CFLAGS_EXTRA=-DMODULE_SMARTPI_ENABLED=1
```
