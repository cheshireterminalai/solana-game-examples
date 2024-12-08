use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod grin_games {
    use super::*;

    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        bet_amount: u64,
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.player1 = ctx.accounts.player.key();
        game.bet_amount = bet_amount;
        game.is_active = true;
        game.bump = *ctx.bumps.get("game").unwrap();
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.player_token_account.to_account_info(),
                    to: ctx.accounts.game_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            bet_amount,
        )?;

        Ok(())
    }

    pub fn join_game(
        ctx: Context<JoinGame>,
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        
        require!(game.is_active, ErrorCode::GameNotActive);
        require!(game.player2.is_none(), ErrorCode::GameFull);
        
        // Store values we need before the first transfer
        let bet_amount = game.bet_amount;
        let bump = game.bump;
        
        // First transfer: player 2's bet
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.player_token_account.to_account_info(),
                    to: ctx.accounts.game_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            bet_amount,
        )?;

        // Set player2 and determine winner
        game.player2 = Some(ctx.accounts.player.key());
        let clock = Clock::get()?;
        let is_player1_winner = clock.unix_timestamp % 2 == 0;
        
        // Calculate total pot
        let total_pot = bet_amount.checked_mul(2).unwrap();
        
        // Determine winner's token account
        let winner_account = if is_player1_winner {
            ctx.accounts.player1_token_account.to_account_info()
        } else {
            ctx.accounts.player_token_account.to_account_info()
        };

        // Transfer pot to winner
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.game_token_account.to_account_info(),
                    to: winner_account,
                    authority: ctx.accounts.game.to_account_info(),
                },
                &[&[
                    b"game",
                    ctx.accounts.player1.key().as_ref(),
                    &[bump],
                ]],
            ),
            total_pot,
        )?;
        
        game.is_active = false;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bet_amount: u64)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = player,
        space = 8 + GameState::SPACE,
        seeds = [b"game", player.key().as_ref()],
        bump
    )]
    pub game: Account<'info, GameState>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, GameState>,
    
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: Used for PDA seeds
    pub player1: AccountInfo<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub player1_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct GameState {
    pub player1: Pubkey,
    pub player2: Option<Pubkey>,
    pub bet_amount: u64,
    pub is_active: bool,
    pub bump: u8,
}

impl GameState {
    pub const SPACE: usize = 32 + 33 + 8 + 1 + 1;
}

#[error_code]
pub enum ErrorCode {
    #[msg("The game is not currently active")]
    GameNotActive,
    #[msg("The game is already full")]
    GameFull,
}
