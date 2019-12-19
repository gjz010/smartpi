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
	
clean:
	cd micropython/ports/unix; make clean
	rm $(TARGET) $(LIBSMARTPI) $(LIBJPEG) $(LIBINFER) -f
