SRCS = $(shell git ls-files '*.go')

all:
	go build
clean:
	go clean
fmt:
	gofmt -w $(SRCS)
fmtcheck:
	@ $(foreach file,$(SRCS),gofmt -s -l $(file);)
