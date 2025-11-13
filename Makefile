
ZIP_OPTIONS := -9j

OUTPUT_DIR := ./build
SDOP_GBA_DIR := ./sdop-gba
SDOP_3DS_DIR := ./sdop-3ds
SDOP_PC_DIR := ./sdop-pc
SDOP_WEB_DIR := ./sdop-web
SDOP_PSP_DIR := ./sdop-psp
SDOP_PICO_DIR := ./sdop-pico
SDOP_C_DIR := ./sdop-c
SDOP_C_TEST_DIR := ./sdop-c-test
SDOP_PS_VITA_DIR := ./sdop-psvita
SDOP_SAVE_EDIT := ./sdop-save-edit

.PHONY: build_all
build_all: build_3ds build_psp build_linux_x86 build_wasm build_pico build_ps_vita build_save_editor_linux_x86

.PHONY: make_output
make_output:
	mkdir -p $(OUTPUT_DIR)

.PHONY: build_docker_image
build_docker_image:
	docker build -t psarda/sdop_builder:latest .

.PHONY: build_with_docker
build_with_docker: build_docker_image
	docker run -v "$(shell pwd):/app" psarda/sdop_builder:latest make build_all

# gba is fully broken right now
.PHONY: build_gba
build_gba: make_output
	cd $(SDOP_GBA_DIR) && cargo build --release
	@agb-gbafix $(SDOP_GBA_DIR)/target/thumbv4t-none-eabi/release/sdop-gba -o $(OUTPUT_DIR)/sdop.gba
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_gba.zip $(OUTPUT_DIR)/sdop.gba
	@rm $(OUTPUT_DIR)/sdop.gba

.PHONY: build_3ds
build_3ds: make_output
	cd $(SDOP_3DS_DIR) && cargo 3ds build --release
	@mv $(SDOP_3DS_DIR)/target/armv6k-nintendo-3ds/release/sdop-3ds.3dsx $(OUTPUT_DIR)/sdop.3dsx
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_3ds.zip $(OUTPUT_DIR)/sdop.3dsx
	@rm $(OUTPUT_DIR)/sdop.3dsx

.PHONY: build_psp
build_psp: make_output
	cd $(SDOP_PSP_DIR) && cargo psp --release
	@mv $(SDOP_PSP_DIR)/target/mipsel-sony-psp/release/EBOOT.PBP $(OUTPUT_DIR)/EBOOT.PBP
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_psp.zip $(OUTPUT_DIR)/EBOOT.PBP
	@rm $(OUTPUT_DIR)/EBOOT.PBP

.PHONY: build_linux_x86
build_linux_x86: make_output
	cd $(SDOP_PC_DIR) && cargo build --release --target=x86_64-unknown-linux-gnu
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_linux_x86.zip ./target/x86_64-unknown-linux-gnu/release/sdop-pc

.PHONY: build_wasm
build_wasm: make_output
	cd $(SDOP_WEB_DIR) && trunk build --release
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop-wasm.zip $(SDOP_WEB_DIR)/dist/*

.PHONY: build_pico
build_pico: make_output
	cd $(SDOP_PICO_DIR) && cargo build --release --target=thumbv8m.main-none-eabihf
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_pico.zip $(SDOP_PICO_DIR)/target/thumbv8m.main-none-eabihf/release/sdop-pico

.PHONY: build_ps_vita
build_ps_vita: make_output
	cd $(SDOP_PS_VITA_DIR) && cargo vita build vpk -- --release
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_ps_vita.zip $(SDOP_PS_VITA_DIR)/target/armv7-sony-vita-newlibeabihf/release/sdop-psvita.vpk


target/release/libsdop_c.a target/release/libsdop_c.d target/release/libsdop_c.so: $(SDOP_C_DIR)/src/lib.rs
	cd $(SDOP_C_DIR) && cargo build --release

.PHONY: build_sdop_c_test
build_sdop_c_test: make_output target/release/libsdop_c.a
	cd $(SDOP_C_TEST_DIR) && gcc main.c ../target/release/libsdop_c.a -o ../build/sdop_c_test

.PHONY: build_save_editor_linux_x86
build_save_editor_linux_x86: make_output
	cd $(SDOP_SAVE_EDIT) && cargo build --release --target=x86_64-unknown-linux-gnu
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_save_editor_linux_x86.zip ./target/x86_64-unknown-linux-gnu/release/sdop-save-edit

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	find . -type d -name target -exec rm -rf {} +

.PHONY: decode_save
decode_save:
	cargo run --manifest-path=$(SDOP_SAVE_EDIT)/Cargo.toml decode --source sdop.sav --unlock-all


.PHONY: encode_save
encode_save:
	cargo run --manifest-path=$(SDOP_SAVE_EDIT)/Cargo.toml encode --source sdop-sav.ron

.PHONY: propagate_save
propagate_save:
	@cp sdop.sav $(SDOP_PICO_DIR)/sdop.sav
	@cp sdop.sav $(SDOP_PS_VITA_DIR)/sdop.sav

.PHONY: load_on_pico
load_on_pico: propagate_save
	cd $(SDOP_PICO_DIR) && cargo run --target=thumbv8m.main-none-eabihf --release
