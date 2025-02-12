mod cpu;

const SAMPLE_MEMORY: [u8; 4096] = [0; 4096];

fn main() {
    let mut test_memory: Vec<u8> = Vec::new();
    test_memory.push(0xA2);
    test_memory.push(0xF0);

    let mut cpu = cpu::CPU::new();
    cpu.initialize();
    cpu.load(test_memory);
    cpu.cycle();
}
