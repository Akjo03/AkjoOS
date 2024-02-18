use std::{
    env,
    process::{self, Command},
};

fn main() {
    println!("BIOS disk image at {}", env!("BIOS_IMAGE"));

    let mut qemu = Command::new(
        format!("{}/tools/qemu/qemu-system-x86_64",
                env::var("CARGO_MANIFEST_DIR").unwrap())
    );

    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));

    qemu.arg("-serial").arg("stdio");

    let accel_enabled = env!("ACCEL_ENABLED").to_string()
        .parse::<bool>().unwrap();

    match (env::consts::OS, accel_enabled) {
        ("windows", true) => {
            qemu.arg("-accel").arg("whpx,kernel-irqchip=off");
            println!("Windows with WHPX enabled.")
        }, ("linux", true) => {
            qemu.arg("-accel").arg("kvm");
            println!("Linux with KVM enabled.")
        }, ("macos", true) => {
            qemu.arg("-accel").arg("hvf");
            println!("macOS with HVF enabled.")
        }, _ => {}
    }

    qemu.arg("-device").arg(format!("VGA,{}", env!("VGA_OPTIONS")));
    println!("VGA options: {}", env!("VGA_OPTIONS"));
    qemu.arg("-m").arg(env!("AVAILABLE_MEMORY"));
    println!("Available memory: {}", env!("AVAILABLE_MEMORY"));

    qemu.arg("-S");

    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}