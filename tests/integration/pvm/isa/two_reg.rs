use vinwolf::types::{Context,  ExitReason};
use vinwolf::pvm::isa::two_reg::{move_reg};

/*#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod test {
        use super::*;
        
        #[test]
        fn test_move_reg() {
            let mut pvm_ctx = Context::default();
            pvm_ctx.reg[2] = 0x5;
            let mut program = ProgramSequence::default();
            program.data = vec![100, 0x00, 100, 0x12, 100, 0xF5, 100, 0x6F];
            program.bitmask = vec![true, false, true, false, true, false, true, false];
            
            move_reg(&mut pvm_ctx, &program).unwrap();
            assert_eq!(pvm_ctx.reg[0], 0);
            assert_eq!(pvm_ctx.pc, 1);

            // Next move instruction
            pvm_ctx.pc += 1;
            move_reg(&mut pvm_ctx, &program).unwrap();
            assert_eq!(pvm_ctx.reg[1], 5);

            // Next move instruction
            pvm_ctx.pc += 1;
            let res = move_reg(&mut pvm_ctx, &program);
            assert_eq!(Err(ExitReason::Panic), res);
            pvm_ctx.pc += 1; // Skip the panic instruction

            // Next move instruction
            pvm_ctx.pc += 1;
            let res = move_reg(&mut pvm_ctx, &program);
            assert_eq!(Err(ExitReason::Panic), res);
        }  
    }  
}*/

