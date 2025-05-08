use askama::Template; // bring trait in scope
use reis_mmiogen::generator;
use reis_mmiogen::mmio;

fn main() {
    let mut ctrl = mmio::Register::new("Ctrl", 0x08001000);
    ctrl.bitfields.push(mmio::Bitfields::new("tx", 1, 0));
    ctrl.bitfields.push(mmio::Bitfields::new("rx", 1, 1));
    ctrl.bitfields.push(mmio::Bitfields::new("nf", 1, 2));
    ctrl.bitfields.push(mmio::Bitfields::new("nco", 16, 16));

    println!(
        "{}",
        generator::cpp::Register { data: ctrl }.render().unwrap()
    );

    let mut uart = mmio::Peripheral::new("Uart");
    uart.registers.push("ctrl");
    uart.registers.push("status");

    println!(
        "{}",
        generator::cpp::Peripheral { data: uart }.render().unwrap()
    );
}
