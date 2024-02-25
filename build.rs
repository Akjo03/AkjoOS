use bootloader::{BootConfig, DiskImageBuilder};
use std::{env, path::PathBuf};
use std::process::Command;

fn main() {
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();

    let os_name = env::var("CARGO_PKG_NAME").unwrap();

    let metadata_out = Command::new("cargo")
        .args(&["metadata", "--format-version", "1", "--no-deps"])
        .output().unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&metadata_out.stdout).unwrap();

    let cpu_count = metadata["packages"][1]["metadata"]["os"]["cpu_count"]
        .as_str().unwrap_or("1");
    let vga_xres = metadata["packages"][1]["metadata"]["os"]["vga_xres"]
        .as_str().unwrap_or("1280");
    let vga_yres = metadata["packages"][1]["metadata"]["os"]["vga_yres"]
        .as_str().unwrap_or("720");
    let vga_mem = metadata["packages"][1]["metadata"]["os"]["vga_mem"]
        .as_str().unwrap_or("64");
    let mem_size = metadata["packages"][1]["metadata"]["os"]["mem_size"]
        .as_str().unwrap_or("128M");
    let accel_enabled = metadata["packages"][1]["metadata"]["os"]["accel_enabled"]
        .as_str().unwrap_or("true");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join(format!("{}-uefi.img", os_name));
    let bios_path = out_dir.join(format!("{}-bios.img", os_name));

    let mut boot_config = BootConfig::default();
    boot_config.frame_buffer_logging = false;

    let mut disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));
    disk_builder.set_boot_config(&boot_config);

    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    let vga_options = format!(
        "vgamem_mb={},xres={},yres={}",
        vga_mem, vga_xres, vga_yres
    );

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
    println!("cargo:rustc-env=CPU_COUNT={}", cpu_count);
    println!("cargo:rustc-env=VGA_OPTIONS={}", vga_options);
    println!("cargo:rustc-env=AVAILABLE_MEMORY={}", mem_size);
    println!("cargo:rustc-env=ACCEL_ENABLED={}", accel_enabled);
}