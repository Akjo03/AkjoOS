use std::{
    env,
    process::{self, Command},
};

fn main() {
    println!("UEFI disk image at {}", env!("UEFI_IMAGE"));

    let mut qemu = Command::new(
        format!("{}/tools/qemu/qemu-system-x86_64",
                env::var("CARGO_MANIFEST_DIR").unwrap())
    );

    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    qemu.arg("-serial").arg("stdio");

    let accel_enabled = env::var("ACCEL_ENABLED").unwrap_or("true".to_string())
        .parse::<bool>().unwrap();

    match (env::consts::OS, accel_enabled) {
        ("windows", true) => {
            qemu.arg("-accel").arg("whpx,kernel-irqchip=off");
        }, ("linux", true) => {
            qemu.arg("-accel").arg("kvm");
        }, ("macos", true) => {
            qemu.arg("-accel").arg("hvf");
        }, _ => {}
    }

    qemu.arg("-device").arg(format!("VGA,{}", env::var("VGA_OPTIONS").unwrap()));

    qemu.arg("-S");

    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}