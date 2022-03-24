CFLAGS = -O2 -g
CXXFLAGS = -O2 -g

objdir = $(OUT_DIR)/test-aux
objpfx = $(objdir)/

all: $(objpfx)memory $(objpfx)threads

$(objdir):
	mkdir -pv $@

$(objpfx)memory: memory.c $(objdir)
	$(CC) $(CFLAGS) $< -o $@

$(objpfx)threads: threads.cc $(objdir)
	$(CXX) $(CXXFLAGS) $< -o $@ -lrt
