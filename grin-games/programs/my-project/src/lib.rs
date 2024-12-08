use anchor_lang::prelude::*;

declare_id!("FAbdppD3ks1Xg3yq2LSmAHETHDjK86kai1gSPVbRzdRX");

#[program]
pub mod my_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
