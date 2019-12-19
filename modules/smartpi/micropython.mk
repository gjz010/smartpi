SMARTPI_MOD_DIR := $(USERMOD_DIR)

# Add all C files to SRC_USERMOD.
SRC_USERMOD += $(SMARTPI_MOD_DIR)/entry.c

# We can add our module folder to include paths if needed
# This is not actually needed in this example.
CFLAGS_USERMOD += -I$(SMARTPI_MOD_DIR) -L /mnt/c/links/embed/smartpi/library -Wl,-Bstatic -Wl,--start-group -lade -lmyriadPlugin -linference_engine_s -lusb -ludev -lngraph -lpugixml -lstb_image -lvpu_common_lib -lvpu_graph_transformer -lmvnc -lfluid -lXLink -lgflags_nothreads -lstdc++ -lc -lgcc -ljpeg -lturbojpeg -lepeg -lexif -ljpeg_sample -linfer_service -lsmartpi -Wl,--end-group
