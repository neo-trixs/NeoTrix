CC = clang
CFLAGS = -dynamiclib -Wall -Werror -O2 -framework CoreFoundation
TARGET = libneotrix_spoof.dylib
INSTALL_DIR = $(HOME)/.neotrix

all: $(TARGET)

$(TARGET): neotrix_spoof.c
	$(CC) $(CFLAGS) -o $(TARGET) neotrix_spoof.c

clean:
	rm -f $(TARGET) *.o

install: $(TARGET)
	mkdir -p $(INSTALL_DIR)
	cp $(TARGET) $(INSTALL_DIR)/$(TARGET)
	@echo "installed to $(INSTALL_DIR)/$(TARGET)"

test: $(TARGET)
	@echo "=== DYLD Interposition Test ==="
	@DYLD_INSERT_LIBRARIES=./$(TARGET) sh -c 'echo "hw.model: $$(sysctl -n hw.model 2>/dev/null)"'
	@DYLD_INSERT_LIBRARIES=./$(TARGET) sh -c 'echo "hostname: $$(hostname 2>/dev/null)"'
	@DYLD_INSERT_LIBRARIES=./$(TARGET) sh -c 'echo "uname -n: $$(uname -n 2>/dev/null)"'
	@DYLD_INSERT_LIBRARIES=./$(TARGET) sh -c 'echo "whoami: $$(whoami 2>/dev/null)"'
	@echo "=== Done ==="

.PHONY: all clean install test
