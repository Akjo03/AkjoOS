use bootloader::{BootConfig, DiskImageBuilder};
use std::{env, path::PathBuf};

fn main() {
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("test_os-uefi.img");
    let bios_path = out_dir.join("test_os-bios.img");

    let mut boot_config = BootConfig::default();
    boot_config.frame_buffer_logging = false;

    let mut disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));
    disk_builder.set_boot_config(&boot_config);

    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    let vga_options = "vgamem_mb=64,xres=1280,yres=720";
    let accel_enabled = "true";

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
    println!("cargo:rustc-env=VGA_OPTIONS={}", vga_options);
    println!("cargo:rustc-env=ACCEL_ENABLED={}", accel_enabled);
}