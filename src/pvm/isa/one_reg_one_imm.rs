

/*fn load_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // One reg one imm -> 04 07 d2 04 | r7 = 0x4d2
    -> Result<(), String> {
    let dest = program.c[pvm_ctx.pc as usize + 1];
    if dest > 13 { return Err("panic".to_string()) };
    let value = get_imm(program, pvm_ctx.pc, ONE_REG_ONE_IMM);
    pvm_ctx.reg[dest as usize] = value as u32;
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);   
    Ok(())
}
    */