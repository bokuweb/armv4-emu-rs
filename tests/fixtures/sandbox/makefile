curdir := $(CURDIR)

build:
	-docker stop arm-gcc
	docker run --rm --name arm-gcc -d -it -v $(curdir):/work/host arm-gcc
	docker exec arm-gcc echo "start build" && mkdir -p built
	docker exec arm-gcc arm-none-eabi-as host/crt.S -o host/built/crt.o
	docker exec arm-gcc arm-none-eabi-ld -t -T host/rom.ld -o host/built/rom.elf host/built/crt.o -Map host/built/rom.map
	docker exec arm-gcc arm-none-eabi-objcopy -O binary host/built/rom.elf host/built/rom.mb
	docker stop arm-gcc
clean:
	rm -rf built
