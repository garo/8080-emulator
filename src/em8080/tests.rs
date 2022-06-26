
use crate::em8080::Em8080;


#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn test_read_byte() {
    let mut sys = Em8080::new();
    sys.memory[0x0000] = 0xAA;
    sys.memory[0x0001] = 0xBB;

    let w = sys.read_word(0);
    assert_eq!(w, 0xBBAA);
}

#[test]
fn test_write_byte() {
    let mut sys = Em8080::new();

    sys.write_word(0, 0xBBAA);
    assert_eq!(sys.memory[0x0000], 0xAA);
    assert_eq!(sys.memory[0x0001], 0xBB);
}

#[test]
fn test_bc() {
    let mut sys = Em8080::new();

    sys.b = 0xAA;
    sys.c = 0xBB;
    assert_eq!(sys.get_bc(), 0xAABB);
    
    sys.set_bc(0xCCDD);
    assert_eq!(sys.b, 0xCC);
    assert_eq!(sys.c, 0xDD);
}

#[test]
fn test_de() {
    let mut sys = Em8080::new();

    sys.d = 0xCC;
    sys.e = 0xDD;
    assert_eq!(sys.get_de(), 0xCCDD);
    
    sys.set_de(0xAABB);
    assert_eq!(sys.d, 0xAA);
    assert_eq!(sys.e, 0xBB);
}

#[test]
fn test_hl() {
    let mut sys = Em8080::new();

    sys.h = 0xCC;
    sys.l = 0xDD;
    assert_eq!(sys.get_hl(), 0xCCDD);
    
    sys.set_hl(0xAABB);
    assert_eq!(sys.h, 0xAA);
    assert_eq!(sys.l, 0xBB);
}

#[test]
fn test_nop() {
    let mut sys = Em8080::new();
    sys.memory[0x0000] = 0x00;
    println!("test_nop: {:#?}", sys);

    let cycles = sys.emulate();
    assert_eq!(cycles, 4);
    assert_eq!(sys.pc, 1);
}

#[test]
fn test_LXI() {
    let mut sys = Em8080::new();

    // LXI B, 0xAABB
    sys.memory[0x0000] = 0x01;
    sys.memory[0x0001] = 0xBB;
    sys.memory[0x0002] = 0xAA;

    // LXI D, 0xCCDD
    sys.memory[0x0003] = 0x11;
    sys.memory[0x0004] = 0xDD;
    sys.memory[0x0005] = 0xCC;

    // LXI H, 0xCCDD
    sys.memory[0x0006] = 0x21;
    sys.memory[0x0007] = 0x02;
    sys.memory[0x0008] = 0x01;

    // LXI SP, 0x1234
    sys.memory[0x0009] = 0x31;
    sys.memory[0x000A] = 0x34;
    sys.memory[0x000B] = 0x12;

    let cycles = sys.emulate();
    println!("test_LXI B:\n{:#?}", sys);
    assert_eq!(sys.pc, 3);
    assert_eq!(cycles, 10);

    let cycles = sys.emulate();
    println!("test_LXI D:\n{:#?}", sys);
    assert_eq!(sys.pc, 6);
    assert_eq!(cycles, 10);

    let cycles = sys.emulate();
    println!("test_LXI D:\n{:#?}", sys);
    assert_eq!(sys.pc, 9);
    assert_eq!(cycles, 10);

    let cycles = sys.emulate();
    println!("test_LXI D:\n{:#?}", sys);
    assert_eq!(sys.pc, 12);
    assert_eq!(cycles, 10);

    assert_eq!(sys.b, 0xAA);
    assert_eq!(sys.c, 0xBB);
    assert_eq!(sys.d, 0xCC);
    assert_eq!(sys.e, 0xDD);
    assert_eq!(sys.h, 0x01);
    assert_eq!(sys.l, 0x02);
    assert_eq!(sys.sp, 0x1234);
}

