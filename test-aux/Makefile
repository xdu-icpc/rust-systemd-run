CFLAGS = -O2 -g -Wall -Wextra -std=gnu17
CXXFLAGS = -O2 -g -Wall -Wextra -std=gnu++17

objdir = $(if $(OUT_DIR), $(OUT_DIR)/test-aux)
objpfx = $(if $(OUT_DIR), $(objdir)/)

targets = $(objpfx)memory $(objpfx)threads $(objpfx)shm $(objpfx)minimal \
		  $(objpfx)floppy.img $(objpfx)rw $(objpfx)rust-systemd-run-test \
		  $(objpfx)nosgid $(objpfx)stdin.txt $(objpfx)waste-fd \
		  $(objpfx)waste-pid $(objpfx)use-stack

all: $(targets) $(if $(OUT_DIR),,.gitignore)

ifeq ($(objdir), )
.PHONY: .gitignore
.gitignore:
	echo $(targets) | sed 's/ /\n/g' > $@
else
$(objdir):
	mkdir -pv $@
endif

$(objpfx)memory: memory.c $(objdir) barrier.h
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

$(objpfx)waste-fd: waste-fd.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@

$(objpfx)waste-pid: waste-pid.cc $(objdir)
	$(CXX) $(CXXFLAGS) $< -o $@ -pthread

$(objpfx)use-stack: use-stack.c $(objdir) barrier.h
	$(CC) $(CFLAGS) $< -o $@

$(objpfx)stdin.txt:
	echo 1145141919810 > $@

# These are only for preventing some file owned by root from showing up in
# the tree.
$(objpfx)floppy.img:
	touch $@

$(objpfx)rust-systemd-run-test:
	touch $@

.PHONY: clean
clean:
	rm -f $(targets)
