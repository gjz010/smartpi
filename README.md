SmartPi Software Stack
========

Software stack for SmartPi, embedded video streaming & neural network classification application based on Raspberry Pi 3.



- [x] Aarch64 Raspberry-Pi Kernel
- [x] Movidius driver for aarch64-linux-musl.
- [x] V4L2-based video streaming.
- [ ] Train and run NN on Myriad.
- [ ] Better replacement for CPython.


Magic spell for compiling statically-linked SmartPI:
========

Every time we make the application more static, we benifit more from it.

```
RUSTFLAGS="-C linker=aarch64-linux-musl-g++" cargo rustc --target aarch64-unknown-linux-musl --release -- -C link-args="-L /home/gjz010/playground/dldt/inference-engine/bin/aarch64/MinSizeRel/lib/  -L /home/gjz010/playground/libusb/usr/local/lib -L /home/gjz010/eudev/usr/lib -L /home/gjz010/playground/dldt/inference-engine/build/lib -Wl,-Bstatic -Wl,--start-group -lade -lmyriadPlugin -linference_engine_s -lusb -ludev -lngraph -lpugixml -lstb_image -lvpu_common_lib -lvpu_graph_transformer -lmvnc -lfluid -lXLink -lgflags_nothreads -lstdc++ -lc -lgcc -ljpeg -lturbojpeg -lepeg -lexif -Wl,--end-group /opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/9.1.0/crtbeginT.o /opt/aarch64-linux-musl-cross/lib/gcc/aarch64-linux-musl/9.1.0/crtend.o"
```