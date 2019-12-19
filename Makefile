TARGET = target/aarch64-unknown-linux-musl/release/micropython
LIBSMARTPI = target/aarch64-unknown-linux-musl/release/libsmartpi.a
CC = $(CROSS_COMPILE)gcc -O3
CXX = $(CROSS_COMPILE)g++ -O3
AR = $(CROSS_COMPILE)ar

MYRIAD_INCLUDE = /home/gjz010/playground/dldt/inference-engine/include
LIBJPEG = library/libjpeg_sample.a
LIBINFER = library/libinfer_service.a

all: $(TARGET)

$(LIBJPEG): c/jpeg_resample.c
	$(CC) c/jpeg_resample.c -c -o c/jpeg_resample.o
	$(AR) rc $(LIBJPEG) c/jpeg_resample.o
$(LIBINFER): c/infer_service.cpp
	$(CXX) c/infer_service.cpp -I $(MYRIAD_INCLUDE) -c -o c/infer_service.o
	$(AR) rc $(LIBINFER) c/infer_service.o

$(TARGET): $(LIBSMARTPI) $(LIBJPEG) $(LIBINFER)
	cd micropython/ports/unix; rm -f micropython; make -j8 deplibs; make -j8 USER_C_MODULES=../../../modules
	cp micropython/ports/unix/micropython $(TARGET)
$(LIBSMARTPI):
	cargo build --release --target=aarch64-unknown-linux-musl
	
sysroot.tar.lzma: $(TARGET)
	tar cvf sysroot.tar -C ./sysroot .
	tar rvf sysroot.tar -C target/aarch64-unknown-linux-musl/release/   micropython
	lzma -e sysroot.tar
smartpi.img: sysroot.tar.lzma
	dd if=/dev/zero of=smartpi.img bs=512 count=45100
	parted smartpi.img -s -a minimal mklabel msdos
	parted smartpi.img -s -a minimal mkpart primary fat16 2048s 45055s
	dd if=/dev/zero of=/tmp/part.img bs=512 count=43008
	mformat -C -i /tmp/part.img -h 21 -t 32 -n 64 -c 1
	mcopy -i /tmp/part.img root/* ::
	mcopy -i /tmp/part.img sysroot.tar.lzma ::/ext/
	dd if=/tmp/part.img of=smartpi.img bs=512 count=43008 seek=2048 conv=notrunc
clean:
	cd micropython/ports/unix; make clean
	rm $(TARGET) $(LIBSMARTPI) $(LIBJPEG) $(LIBINFER) -f
