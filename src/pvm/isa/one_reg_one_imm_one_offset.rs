

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

/*fn get_imm(program: &ProgramSequence, pc: u32, instr_type: usize) -> u32 {
    let mut i = pc; 
    let l_x = match instr_type {
        ONE_REG_ONE_IMM | 
        TWO_REG_ONE_IMM => { 
                            i += 2; 
                            //println!("TWO_REG_ONE_IMM");
                            let x: isize = skip(pc, &program.k).saturating_sub(1) as isize;
                            let x_u32 = if x < 0 { 0 } else { x as u32 }; 
                            std::cmp::min(4_u32, x_u32)
                        },
        ONE_REG_ONE_IMM_ONE_OFFSET => {
                            i += 2;
                            //println!("ONE_REG_ONE_IMM_ONE_OFFSET");
                            std::cmp::min(4_u32, (program.c[pc as usize + 1] / 16) as u32)
        },
        _ => return 0,
    };
    //println!("lx = {l_x}");
    return extend_sign(&program.c[i as usize ..i as usize + l_x as usize].to_vec(), (4 - l_x as usize) as u32);
}*/