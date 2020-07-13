use r8::cpu;

fn main() {
    let mut c = cpu::CPU::new();
    c.step();
    println!("Hello, world!");
}
