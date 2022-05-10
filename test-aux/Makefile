CFLAGS = -O2 -g -Wall -Wextra
CXXFLAGS = -O2 -g -Wall -Wextra

objdir = $(if $(OUT_DIR), $(OUT_DIR)/test-aux)
objpfx = $(if $(OUT_DIR), $(objdir)/)

targets = $(objpfx)memory $(objpfx)threads $(objpfx)shm $(objpfx)minimal \
		  $(objpfx)floppy.img $(objpfx)rw $(objpfx)rust-systemd-run-test \
		  $(objpfx)nosgid

all: $(targets) $(if $(OUT_DIR),,.gitignore)

ifeq ($(objdir), )
.PHONY: .gitignore
.gitignore:
	echo $(targets) | sed 's/ /\n/g' > $@
else
$(objdir):
	mkdir -pv $@
endif

$(objpfx)memory: memory.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@

$(objpfx)threads: threads.cc $(objdir)
	$(CXX) $(CXXFLAGS) $< -o $@ -lrt -pthread

$(objpfx)shm: shm.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@

$(objpfx)minimal: minimal.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@ -static -nostdlib -Wl,--as-needed -lc -Wl,--no-as-needed

$(objpfx)rw: rw.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@

$(objpfx)nosgid: nosgid.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@
	chmod g+s $@

# These are only for preventing some file owned by root from showing up in
# the tree.
$(objpfx)floppy.img:
	touch $@

$(objpfx)rust-systemd-run-test:
	touch $@

.PHONY: clean
clean:
	rm -f $(targets)
