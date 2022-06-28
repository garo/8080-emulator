use std::{num::ParseIntError};

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
fn test_lxi() {
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

// Takes a string such as "AABB" and returns Vec with AA and BB
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

// Runs a single operation passed as a string of opcodes
fn run_op(sys : &mut Em8080, command : &str) -> u64{

    let result = decode_hex(command).unwrap();

    for (i, x) in result.iter().enumerate() {
        sys.memory[i as usize] = *x;
    }
    sys.pc = 0;

    sys.emulate()
}

#[test]
fn test_mvi() {
    let mut sys = Em8080::new();

    run_op(&mut sys, "3EAA");
    assert_eq!(sys.a, 0xAA);

    run_op(&mut sys, "06BB");
    assert_eq!(sys.b, 0xBB);

    run_op(&mut sys, "0ECC");
    assert_eq!(sys.c, 0xCC);

    run_op(&mut sys, "16DD");
    assert_eq!(sys.d, 0xDD);

    run_op(&mut sys, "1EEE");
    assert_eq!(sys.e, 0xEE);

    run_op(&mut sys, "2601");
    assert_eq!(sys.h, 0x01);

    run_op(&mut sys, "2E02");
    assert_eq!(sys.l, 0x02);

    // MVI M writes value 03 to memory location specified by H,L
    run_op(&mut sys, "3603");
    assert_eq!(sys.memory[0x0102], 0x03);
}

#[test]
fn test_mov() {
    let mut sys = Em8080::new();

    sys.b = 1;
    run_op(&mut sys, "40"); // MOV B, B
    assert_eq!(sys.b, 0x01);

    sys.c = 2;
    run_op(&mut sys, "41"); // MOV B, C
    assert_eq!(sys.b, 0x02);

    sys.d = 3;
    run_op(&mut sys, "42"); // MOV B, D
    assert_eq!(sys.b, 0x03);

    sys.e = 4;
    run_op(&mut sys, "43"); // MOV B, E
    assert_eq!(sys.b, 0x04);

    sys.h = 5;
    run_op(&mut sys, "44"); // MOV B, H
    assert_eq!(sys.b, 0x05);

    sys.l = 6;
    run_op(&mut sys, "45"); // MOV B, L
    assert_eq!(sys.b, 0x06);

    sys.memory[0x0506] = 0xAA;
    run_op(&mut sys, "46"); // MOV B, M
    assert_eq!(sys.b, 0xAA);

    sys.a = 7;
    run_op(&mut sys, "47"); // MOV B, A
    assert_eq!(sys.b, 0x07);



    sys.b = 1;
    run_op(&mut sys, "48"); // MOV C, B
    assert_eq!(sys.c, 0x01);

    sys.c = 2;
    run_op(&mut sys, "49"); // MOV C, C
    assert_eq!(sys.c, 0x02);

    sys.d = 3;
    run_op(&mut sys, "4A"); // MOV C, D
    assert_eq!(sys.c, 0x03);

    sys.e = 4;
    run_op(&mut sys, "4B"); // MOV C, E
    assert_eq!(sys.c, 0x04);

    sys.h = 5;
    run_op(&mut sys, "4C"); // MOV C, H
    assert_eq!(sys.c, 0x05);

    sys.l = 6;
    run_op(&mut sys, "4D"); // MOV C, L
    assert_eq!(sys.c, 0x06);

    sys.memory[0x0506] = 0xAA;
    run_op(&mut sys, "4E"); // MOV C, M
    assert_eq!(sys.c, 0xAA);

    sys.a = 7;
    run_op(&mut sys, "4F"); // MOV C, A
    assert_eq!(sys.c, 0x07);    


    // ROW 2

    sys.b = 1;
    run_op(&mut sys, "50"); // MOV D, B
    assert_eq!(sys.d, 0x01);

    sys.c = 2;
    run_op(&mut sys, "51"); // MOV D, C
    assert_eq!(sys.d, 0x02);

    sys.d = 3;
    run_op(&mut sys, "52"); // MOV D, D
    assert_eq!(sys.d, 0x03);

    sys.e = 4;
    run_op(&mut sys, "53"); // MOV D, E
    assert_eq!(sys.d, 0x04);

    sys.h = 5;
    run_op(&mut sys, "54"); // MOV D, H
    assert_eq!(sys.d, 0x05);

    sys.l = 6;
    run_op(&mut sys, "55"); // MOV D, L
    assert_eq!(sys.d, 0x06);

    sys.memory[0x0506] = 0xAA;
    run_op(&mut sys, "56"); // MOV D, M
    assert_eq!(sys.d, 0xAA);

    sys.a = 7;
    run_op(&mut sys, "57"); // MOV D, A
    assert_eq!(sys.d, 0x07);



    sys.b = 1;
    run_op(&mut sys, "58"); // MOV E, B
    assert_eq!(sys.e, 0x01);

    sys.c = 2;
    run_op(&mut sys, "59"); // MOV E, C
    assert_eq!(sys.e, 0x02);

    sys.d = 3;
    run_op(&mut sys, "5A"); // MOV E, D
    assert_eq!(sys.e, 0x03);

    sys.e = 4;
    run_op(&mut sys, "5B"); // MOV E, E
    assert_eq!(sys.e, 0x04);

    sys.h = 5;
    run_op(&mut sys, "5C"); // MOV E, H
    assert_eq!(sys.e, 0x05);

    sys.l = 6;
    run_op(&mut sys, "5D"); // MOV E, L
    assert_eq!(sys.e, 0x06);

    sys.memory[0x0506] = 0xAA;
    run_op(&mut sys, "5E"); // MOV E, M
    assert_eq!(sys.e, 0xAA);

    sys.a = 7;
    run_op(&mut sys, "5F"); // MOV E, A
    assert_eq!(sys.e, 0x07);    

    // ROW 3

    sys.b = 1;
    run_op(&mut sys, "60"); // MOV H, B
    assert_eq!(sys.h, 0x01);

    sys.c = 2;
    run_op(&mut sys, "61"); // MOV H, C
    assert_eq!(sys.h, 0x02);

    sys.d = 3;
    run_op(&mut sys, "62"); // MOV H, D
    assert_eq!(sys.h, 0x03);

    sys.e = 4;
    run_op(&mut sys, "63"); // MOV H, E
    assert_eq!(sys.h, 0x04);

    sys.h = 5;
    run_op(&mut sys, "64"); // MOV H, H
    assert_eq!(sys.h, 0x05);

    sys.l = 6;
    run_op(&mut sys, "65"); // MOV H, L
    assert_eq!(sys.h, 0x06);

    sys.h = 0x05;
    sys.l = 0x06;
    sys.memory[0x0506] = 0xBB;
    run_op(&mut sys, "66"); // MOV H, M
    assert_eq!(sys.h, 0xBB);

    sys.a = 7;
    run_op(&mut sys, "67"); // MOV H, A
    assert_eq!(sys.h, 0x07);



    sys.b = 1;
    run_op(&mut sys, "68"); // MOV L, B
    assert_eq!(sys.l, 0x01);

    sys.c = 2;
    run_op(&mut sys, "69"); // MOV L, C
    assert_eq!(sys.l, 0x02);

    sys.d = 3;
    run_op(&mut sys, "6A"); // MOV L, D
    assert_eq!(sys.l, 0x03);

    sys.e = 4;
    run_op(&mut sys, "6B"); // MOV L, E
    assert_eq!(sys.l, 0x04);

    sys.h = 5;
    run_op(&mut sys, "6C"); // MOV L, H
    assert_eq!(sys.l, 0x05);

    sys.l = 6;
    run_op(&mut sys, "6D"); // MOV L, L
    assert_eq!(sys.l, 0x06);

    sys.memory[0x0506] = 0xAA;
    run_op(&mut sys, "6E"); // MOV L, M
    assert_eq!(sys.l, 0xAA);

    sys.a = 7;
    run_op(&mut sys, "6F"); // MOV L, A
    assert_eq!(sys.l, 0x07);

    // Row 4

    sys.h = 0x10;
    sys.l = 0x20;
    sys.b = 1;
    run_op(&mut sys, "70"); // MOV M, B
    assert_eq!(sys.memory[0x1020], 0x01);

    sys.c = 2;
    run_op(&mut sys, "71"); // MOV M, C
    assert_eq!(sys.memory[0x1020], 0x02);

    sys.d = 3;
    run_op(&mut sys, "72"); // MOV M, D
    assert_eq!(sys.memory[0x1020], 0x03);

    sys.e = 4;
    run_op(&mut sys, "73"); // MOV M, E
    assert_eq!(sys.memory[0x1020], 0x04);

    sys.h = 0x10;
    sys.l = 0x20;
    run_op(&mut sys, "74"); // MOV M, H
    assert_eq!(sys.memory[0x1020], 0x10);

    sys.h = 0x10;
    sys.l = 0x20;
    run_op(&mut sys, "75"); // MOV M, L
    assert_eq!(sys.memory[0x1020], 0x20);

    sys.h = 0x10;
    sys.l = 0x20;
    sys.a = 7;
    run_op(&mut sys, "77"); // MOV M, A
    assert_eq!(sys.memory[0x1020], 0x07);


    sys.b = 1;
    run_op(&mut sys, "78"); // MOV A, B
    assert_eq!(sys.a, 0x01);

    sys.c = 2;
    run_op(&mut sys, "79"); // MOV A, C
    assert_eq!(sys.a, 0x02);

    sys.d = 3;
    run_op(&mut sys, "7A"); // MOV A, D
    assert_eq!(sys.a, 0x03);

    sys.e = 4;
    run_op(&mut sys, "7B"); // MOV A, E
    assert_eq!(sys.a, 0x04);

    sys.h = 5;
    run_op(&mut sys, "7C"); // MOV A, H
    assert_eq!(sys.a, 0x05);

    sys.l = 6;
    run_op(&mut sys, "7D"); // MOV A, L
    assert_eq!(sys.a, 0x06);

    sys.memory[0x0506] = 0xAA;
    run_op(&mut sys, "7E"); // MOV A, M
    assert_eq!(sys.a, 0xAA);

    sys.a = 7;
    run_op(&mut sys, "7F"); // MOV A, A
    assert_eq!(sys.a, 0x07);

}

/*
#[test]
fn test_add() {
    let mut sys = Em8080::new();
    
    sys.add(&mut self.a, 1, 0);
    println!("add: {}", sys.a);

    sys.a = 5;
    sys.b = 2;

    //run_op(&mut sys, "80"); // ADD B
    //assert_eq!(sys.a, 0x07);

}
*/

#[test]
fn test_inr() {
    let mut sys = Em8080::new();
    assert_eq!(sys.inr(255), 0);
    assert_eq!(sys.flags.zero, true);

    run_op(&mut sys, "04"); // INR B
    assert_eq!(sys.b, 0x01);

    run_op(&mut sys, "0C"); // INR C
    assert_eq!(sys.c, 0x01);

    run_op(&mut sys, "14"); // INR D
    assert_eq!(sys.d, 0x01);

    run_op(&mut sys, "1C"); // INR E
    assert_eq!(sys.d, 0x01);

    run_op(&mut sys, "24"); // INR H
    assert_eq!(sys.h, 0x01);

    run_op(&mut sys, "2C"); // INR L
    assert_eq!(sys.h, 0x01);

    sys.h = 0x10;
    sys.l = 0x20;
    run_op(&mut sys, "34"); // INR M
    assert_eq!(sys.memory[0x1020], 0x01);

    run_op(&mut sys, "3C"); // INR A
    assert_eq!(sys.a, 0x01);

}

#[test]
fn test_dcr() {
    let mut sys = Em8080::new();
    assert_eq!(sys.dcr(255), 254);
    assert_eq!(sys.flags.zero, false);
    assert_eq!(sys.dcr(1), 0);
    assert_eq!(sys.flags.zero, true);

    run_op(&mut sys, "05"); // DCR B
    assert_eq!(sys.b, 0xFF);

    run_op(&mut sys, "0D"); // DCR C
    assert_eq!(sys.c, 0xFF);

    run_op(&mut sys, "15"); // DCR D
    assert_eq!(sys.d, 0xFF);

    run_op(&mut sys, "1D"); // DCR E
    assert_eq!(sys.d, 0xFF);

    run_op(&mut sys, "25"); // DCR H
    assert_eq!(sys.h, 0xFF);

    run_op(&mut sys, "2D"); // DCR L
    assert_eq!(sys.h, 0xFF);

    sys.h = 0x10;
    sys.l = 0x20;
    run_op(&mut sys, "35"); // DCR M
    assert_eq!(sys.memory[0x1020], 0xFF);

    run_op(&mut sys, "3D"); // DCR A
    assert_eq!(sys.a, 0xFF);

}

#[test]
fn test_inx() {
    let mut sys = Em8080::new();
    sys.set_bc(0xFF);
    run_op(&mut sys, "03"); // INX B
    assert_eq!(sys.get_bc(), 0x100);
    
    sys.set_de(0xFF);
    run_op(&mut sys, "13"); // INX D
    assert_eq!(sys.get_de(), 0x100);
    
    sys.set_hl(0xFF);
    run_op(&mut sys, "23"); // INX H
    assert_eq!(sys.get_hl(), 0x100);
    
    sys.sp = 0xFF;
    run_op(&mut sys, "33"); // INX SP
    assert_eq!(sys.sp, 0x100);

}

#[test]
fn test_dcx() {
    let mut sys = Em8080::new();
    sys.set_bc(0x100);
    run_op(&mut sys, "0B"); // DCX B
    assert_eq!(sys.get_bc(), 0xFF);
    
    sys.set_de(0x100);
    run_op(&mut sys, "1B"); // DCX D
    assert_eq!(sys.get_de(), 0xFF);
    
    sys.set_hl(0x100);
    run_op(&mut sys, "2B"); // DCX H
    assert_eq!(sys.get_hl(), 0xFF);
    
    sys.sp = 0x100;
    run_op(&mut sys, "3B"); // DCX SP
    assert_eq!(sys.sp, 0xFF);
}

#[test]
fn test_add() {
    let mut sys = Em8080::new();

    sys.a = 1;
    sys.b = 1;
    run_op(&mut sys, "80"); // ADD B
    assert_eq!(sys.a, 0x02);
    
    sys.c = 1;
    run_op(&mut sys, "81"); // ADD C
    assert_eq!(sys.a, 0x03);
    
    sys.d = 1;
    run_op(&mut sys, "82"); // ADD D
    assert_eq!(sys.a, 0x04);
    
    sys.e = 1;
    run_op(&mut sys, "83"); // ADD E
    assert_eq!(sys.a, 0x05);
    
    sys.h = 1;
    run_op(&mut sys, "84"); // ADD H
    assert_eq!(sys.a, 0x06);
    
    sys.l = 1;
    run_op(&mut sys, "85"); // ADD L
    assert_eq!(sys.a, 0x07);
    
    sys.memory[0x0101] = 1;
    run_op(&mut sys, "86"); // ADD M
    assert_eq!(sys.a, 0x08);
    
    run_op(&mut sys, "87"); // ADD A
    assert_eq!(sys.a, 0x10);
}

#[test]
fn test_adc() {
    let mut sys = Em8080::new();

    sys.flags.carry = true;
    sys.a = 1;
    sys.b = 1;
    run_op(&mut sys, "88"); // ADC B
    assert_eq!(sys.a, 0x03);

    sys.flags.carry = true;
    sys.c = 1;
    run_op(&mut sys, "89"); // ADC C
    assert_eq!(sys.a, 0x05);
    
    sys.flags.carry = true;
    sys.d = 1;
    run_op(&mut sys, "8A"); // ADC D
    assert_eq!(sys.a, 0x07);
    
    sys.flags.carry = true;
    sys.e = 1;
    run_op(&mut sys, "8B"); // ADC E
    assert_eq!(sys.a, 0x09);
    
    sys.flags.carry = true;
    sys.h = 1;
    run_op(&mut sys, "8C"); // ADC H
    assert_eq!(sys.a, 0x0B);
    
    sys.flags.carry = true;
    sys.l = 1;
    run_op(&mut sys, "8D"); // ADC L
    assert_eq!(sys.a, 0x0D);
    
    sys.flags.carry = true;
    sys.memory[0x0101] = 1;
    run_op(&mut sys, "8E"); // ADC M
    assert_eq!(sys.a, 0x0F);
    
    sys.flags.carry = true;
    run_op(&mut sys, "8F"); // ADC A
    assert_eq!(sys.a, 0x1F);
}

#[test]
fn test_sub() {
    let mut sys = Em8080::new();

    sys.a = 10;
    sys.b = 1;
    run_op(&mut sys, "90"); // SUB B
    assert_eq!(sys.a, 0x09);
    
    sys.c = 1;
    run_op(&mut sys, "91"); // SUB C
    assert_eq!(sys.a, 0x08);
    
    sys.d = 1;
    run_op(&mut sys, "92"); // SUB D
    assert_eq!(sys.a, 0x07);
    
    sys.e = 1;
    run_op(&mut sys, "93"); // SUB E
    assert_eq!(sys.a, 0x06);
    
    sys.h = 1;
    run_op(&mut sys, "94"); // SUB H
    assert_eq!(sys.a, 0x05);
    
    sys.l = 1;
    run_op(&mut sys, "95"); // SUB L
    assert_eq!(sys.a, 0x04);
    
    sys.memory[0x0101] = 1;
    run_op(&mut sys, "96"); // SUB M
    assert_eq!(sys.a, 0x03);
    
    run_op(&mut sys, "97"); // SUB A
    assert_eq!(sys.a, 0x00);
}

#[test]
fn test_sbb() {
    let mut sys = Em8080::new();

    sys.a = 4;
    sys.b = 2;
    sys.flags.carry = true;
    run_op(&mut sys, "98"); // SBB B
    assert_eq!(sys.a, 0x01);
}

#[test]
fn test_ana() {
    let mut sys = Em8080::new();

    sys.a = 0xFC;
    sys.b = 0xF;
    sys.flags.carry = true;
    run_op(&mut sys, "A0"); // ANA B
    assert_eq!(sys.a, 0x0C);
}

#[test]
fn test_xra() {
    let mut sys = Em8080::new();

    sys.a = 0x5C;
    sys.b = 0x78;
    sys.flags.carry = true;
    run_op(&mut sys, "A8"); // XRA B
    assert_eq!(sys.a, 0x24);
}


#[test]
fn test_ora() {
    let mut sys = Em8080::new();

    sys.a = 0x33;
    sys.b = 0x0F;
    sys.flags.carry = true;
    run_op(&mut sys, "B0"); // ORA B
    assert_eq!(sys.a, 0x3F);
}

#[test]
fn test_cmp() {
    let mut sys = Em8080::new();

    sys.a = 0x0A;
    sys.e = 0x05;
    run_op(&mut sys, "BB"); // CMP E
    //println!("CMP 0xA vs 0x5:\n{:#?}", sys);
    assert_eq!(sys.flags.carry, false);
    assert_eq!(sys.flags.zero, false);

    sys.a = 0x0A;
    sys.e = 0x0A;
    run_op(&mut sys, "BB"); // CMP E
    //println!("CMP 0xA vs 0xA:\n{:#?}", sys);
    assert_eq!(sys.flags.zero, true);

    sys.a = 0x0A;
    sys.e = 0x0F;
    run_op(&mut sys, "BB"); // CMP E
    //println!("CMP 0xA vs 0xF:\n{:#?}", sys);
    assert_eq!(sys.flags.carry, true);
    assert_eq!(sys.flags.zero, false);

}

#[test]
fn test_jnz() {
    let mut sys = Em8080::new();

    sys.flags.zero = false;
    run_op(&mut sys, "C2FFAA");
    assert_eq!(sys.pc, 0xAAFF);

    sys.flags.zero = true;
    run_op(&mut sys, "C21010");
    assert_eq!(sys.pc, 0x03);
}

#[test]
fn test_jnc() {
    let mut sys = Em8080::new();

    sys.flags.carry = false;
    run_op(&mut sys, "D2FFAA");
    assert_eq!(sys.pc, 0xAAFF);

    sys.flags.carry = true;
    run_op(&mut sys, "D21010");
    assert_eq!(sys.pc, 0x03);
}

#[test]
fn test_jpo() {
    let mut sys = Em8080::new();

    sys.flags.parity = false;
    run_op(&mut sys, "E2FFAA");
    assert_eq!(sys.pc, 0xAAFF);

    sys.flags.parity = true;
    run_op(&mut sys, "E21010");
    assert_eq!(sys.pc, 0x03);
}

#[test]
fn test_jp() {
    let mut sys = Em8080::new();

    sys.flags.sign = false;
    run_op(&mut sys, "F2FFAA");
    assert_eq!(sys.pc, 0xAAFF);

    sys.flags.sign = true;
    run_op(&mut sys, "F21010");
    assert_eq!(sys.pc, 0x03);
}

#[test]
fn test_jmp() {
    let mut sys = Em8080::new();

    run_op(&mut sys, "C3FFAA");
    assert_eq!(sys.pc, 0xAAFF);
}

#[test]
fn test_cnz() {
    let mut sys = Em8080::new();

    sys.sp = 0x4000;
    sys.flags.zero = false;
    run_op(&mut sys, "C4FFAA");
    assert_eq!(sys.pc, 0xAAFF);
    assert_eq!(sys.sp, 0x3FFE);
}

#[test]
fn test_cnc() {
    let mut sys = Em8080::new();
    
    sys.sp = 0x4000;
    sys.flags.carry = false;
    run_op(&mut sys, "D4FFAA");
    assert_eq!(sys.pc, 0xAAFF);
    assert_eq!(sys.sp, 0x3FFE);
}

#[test]
fn test_cpo() { // Call if Parity Odd
    let mut sys = Em8080::new();
    
    sys.sp = 0x4000;
    // This number has two 1 in binary, so Parity Odd
    sys.flags.set_all_but_carry(4);
    run_op(&mut sys, "E4FFAA");
    assert_eq!(sys.pc, 0xAAFF);
    assert_eq!(sys.sp, 0x3FFE);

    sys.sp = 0x4000;
    // This number has just one 1 in binary, so Parity Odd
    sys.flags.set_all_but_carry(3);
    run_op(&mut sys, "E4FFAA");
    assert_eq!(sys.pc, 0x03);
    assert_eq!(sys.sp, 0x4000);

}

#[test]
fn test_cp() { // Call if Plus
    let mut sys = Em8080::new();
    
    
    // positive value
    sys.sp = 0x4000;
    sys.flags.set_all_but_carry(5);
    run_op(&mut sys, "F4FFAA");
    assert_eq!(sys.pc, 0xAAFF);
    assert_eq!(sys.sp, 0x3FFE);

    // negative value (highest bit set)
    sys.sp = 0x4000;
    sys.flags.set_all_but_carry(255);
    run_op(&mut sys, "F4FFAA");
    assert_eq!(sys.pc, 0x03);
    assert_eq!(sys.sp, 0x4000);
}

#[test]
fn test_push() {
    let mut sys = Em8080::new();
    
    sys.b = 0xAA;
    sys.c = 0xBB;
    sys.sp = 0x4000;
    run_op(&mut sys, "C5");
    assert_eq!(sys.memory[0x3FFF], 0xAA);
    assert_eq!(sys.memory[0x3FFE], 0xBB);


    sys.a = 0xAA;
    sys.flags.carry = true;
    sys.flags.zero = true;
    sys.flags.parity = true;
    sys.sp = 0x4000;
    run_op(&mut sys, "F5");
    assert_eq!(sys.memory[0x3FFF], 0xAA);
    assert_eq!(sys.memory[0x3FFE], 0x45);
}

#[test]
fn test_pop() {
    let mut sys = Em8080::new();
    
    sys.sp = 0x4000 - 2;
    sys.memory[0x3FFF] = 0xAA;
    sys.memory[0x3FFE] = 0xBB;
    run_op(&mut sys, "C1");
    assert_eq!(sys.b, 0xAA);
    assert_eq!(sys.c, 0xBB);
}

#[test]
fn test_xthl() {
    let mut sys = Em8080::new();
    
    sys.sp = 0x10AD;
    sys.memory[0x10AC] = 0xFF;
    sys.memory[0x10AD] = 0xF0;
    sys.memory[0x10AE] = 0x0D;
    sys.memory[0x10AF] = 0xFF;
    sys.h = 0x0B;
    sys.l = 0x3C;
    run_op(&mut sys, "E3");
    assert_eq!(sys.h, 0x0D);
    assert_eq!(sys.l, 0xF0);
    assert_eq!(sys.memory[0x10AD], 0x3C);
    assert_eq!(sys.memory[0x10AE], 0x0B);
}

#[test]
fn test_adi() {
    let mut sys = Em8080::new();
    
    sys.a = 0x1;
    run_op(&mut sys, "C602");
    assert_eq!(sys.a, 0x03);
}

#[test]
fn test_sui() {
    let mut sys = Em8080::new();
    
    sys.a = 0x4;
    run_op(&mut sys, "D602");
    assert_eq!(sys.a, 0x02);
}

#[test]
fn test_ani() {
    let mut sys = Em8080::new();
    
    sys.a = 0x3A;
    run_op(&mut sys, "E60F");
    assert_eq!(sys.a, 0x0A);
}