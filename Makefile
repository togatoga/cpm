SRCS = $(shell git ls-files '*.go')

all:
	go build
clean:
	go clean
fmt:
	gofmt -w $(SRCS)
