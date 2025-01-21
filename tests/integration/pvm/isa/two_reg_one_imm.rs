use vinwolf::types::{Context,  ExitReason};
use vinwolf::pvm::isa::two_reg_one_imm::{add_imm_32};

/*#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod test {
        use super::*;
        
        //#[test]
        fn test_add_imm_32() {
            let mut pvm_ctx = Context::default();
            pvm_ctx.reg[0] = 1;
            let mut program = ProgramSequence::default();
            program.data = vec![131, 0x78, 0];
            program.bitmask = vec![true, false, false];
            
            add_imm_32(&mut pvm_ctx, &program).unwrap();
            assert_eq!(pvm_ctx.reg[0], 0);
        }  
    }  
}*/
