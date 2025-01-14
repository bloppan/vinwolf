


/*fn add_imm_32(pvm_ctx: &mut PVM, program: &ProgramSequence) // Two regs one imm -> 02 79 02 | r9 = r7 + 0x2
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let value = get_imm(program, pvm_ctx.pc, TWO_REG_ONE_IMM);
    //println!("value = {value}");
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize].wrapping_add(value);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn add(pvm_ctx: &mut PVM, program: &ProgramSequence) // Three regs -> 08 87 09 | r9 = r7 + r8
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) }; 
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize].wrapping_add(pvm_ctx.reg[b as usize]);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and(pvm_ctx: &mut PVM, program: &ProgramSequence) // Three regs -> 17 87 09 | r9 = r7 & r8
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize] & pvm_ctx.reg[b as usize];
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // Two regs one imm -> 12 79 03 | r9 = r7 & 0x3
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    let value = get_imm(program, pvm_ctx.pc, TWO_REG_ONE_IMM);
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize] & value;
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}
    */