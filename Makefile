
ZIP_OPTIONS := -9j

OUTPUT_DIR := ./build
SDOP_GBA_DIR := ./sdop-gba
SDOP_3DS_DIR := ./sdop-3ds
SDOP_PC_DIR := ./sdop-pc
SDOP_WEB_DIR := ./sdop-web
SDOP_PSP_DIR := ./sdop-psp
SDOP_PICO_DIR := ./sdop-pico
SDOP_SAVE_EDIT := ./sdop-save-edit

.PHONY: build_all
build_all: build_gba build_3ds build_psp build_linux_x86 build_wasm build_pico

.PHONY: make_output
make_output:
	mkdir -p $(OUTPUT_DIR)

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
	@mv $(SDOP_PSP_DIR)/target/mipsel-sony-psp/release/EBOOT.PBP $(OUTPUT_DIR)/sdop.PBP
	@zip $(ZIP_OPTIONS) $(OUTPUT_DIR)/sdop_psp.zip $(OUTPUT_DIR)/sdop.PBP
	@rm $(OUTPUT_DIR)/sdop.PBP

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

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	find . -type d -name target -exec rm -rf {} +

.PHONY: decode_save
decode_save:
	cargo run --manifest-path=$(SDOP_SAVE_EDIT)/Cargo.toml decode --source sdop.sav


.PHONY: encode_save
encode_save:
	cargo run --manifest-path=$(SDOP_SAVE_EDIT)/Cargo.toml encode --source sdop-sav.ron
