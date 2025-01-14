

/*fn branch_eq_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // One reg, one imm, one offset -> 07 27 d3 04 | jump if r7 = 0x4d3
    -> Result<(), String> {
    let target = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if target > 13 { return Err("panic".to_string()) };
    let value = get_imm(program, pvm_ctx.pc, ONE_REG_ONE_IMM_ONE_OFFSET);
    //println!("value branch = {value}");
    if pvm_ctx.reg[target as usize] == value as u32 {
        pvm_ctx.pc = basic_block_seq(pvm_ctx.pc, &program.k);
    } 
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}*/