use anchor_lang::prelude::*;
// use anchor_lang::solana_program::{pubkey::Pubkey};
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use chainlink_solana as chainlink;
declare_id!("5AoXpoEbwgDJSov7aePF2ofiMVBNsFhBMJkqjpMXWWGX");

// program & feed in devnet
// pub const CHAINLINK_PROGRAM_ID: &str = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
// pub const SOL_USD_FEED: &str = "HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6";

// write anchor program for: - game Bravencia cần payment qua chain Solana
// - token Bravancia có tên là BVC
// - rate: 1 USD = 10 BVC hay 1 BVC = 0.1 USD
// Yêu cầu: a cần 1 program có các yêu cầu sau:
// - 1 function depositUSDC: user gửi USDC qua program, và program gửi USDC tới admin-wallet và cũng bắn events
// - 1 function depositSOL: user sẽ gửi SOL qua program, và program gửi SOl tới admin-wallet, sau đó bắn events.
// Lưu ý vụ quy đổi rate SOL ra USD, rồi từ USD ra BVC
// Lưu ý bắn events thì cần đẩy đủ thông tin chút để system biết cộng balance BVC cho wallet nào vào Database
#[program]
pub mod bravencia_program {
    use super::*;

    // Deposit USDC and emit event with BVC equivalent
    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> Result<()> {
        // Transfer USDC from user to admin
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.admin_usdc_account.to_account_info(),
            authority: ctx.accounts.user_wallet.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        // Calculate BVC amount (1 USDC = 1 USD = 10 BVC)
        let usd_value = amount as f64 / 1_000_000.0; // USDC has 6 decimals
        let bvc_amount = (usd_value * 10.0).round() as u64;

        // Emit deposit event
        emit_deposit_event(DepositEvent {
            user_wallet: ctx.accounts.user_wallet.key(),
            deposit_amount: amount,
            deposit_currency: "USDC".to_string(),
            usd_value,
            bvc_amount,
            admin_wallet: ctx.accounts.admin_usdc_account.owner,
            tx_signature: ctx.accounts.user_wallet.key().to_string(), // Simplified for example
        })?;

        Ok(())
    }

    // Deposit SOL and emit event with BVC equivalent
    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        // Transfer SOL from user to admin
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.user_wallet.key,
            ctx.accounts.admin_wallet.key,
            amount,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.user_wallet.to_account_info(), // from account.clone
                ctx.accounts.admin_wallet.to_account_info(), // to account.clone
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let round = chainlink::latest_round_data(
            ctx.accounts.chainlink_program.to_account_info(),
            ctx.accounts.chainlink_feed.to_account_info(),
        )?;

        // Price is returned with 8 decimal places
        let sol_price = round.answer as f64 / 1_000_000_00.0;
        // Calculate USD value and BVC amount
        let sol_amount = amount as f64 / 1_000_000_000.0; // SOL has 9 decimals
        let usd_value = sol_amount * sol_price;
        let bvc_amount = (usd_value * 10.0).round() as u64; // 10 BVC per 1 USD

        // Emit deposit event
        emit_deposit_event(DepositEvent {
            user_wallet: ctx.accounts.user_wallet.key(),
            deposit_amount: amount,
            deposit_currency: "SOL".to_string(),
            usd_value,
            bvc_amount,
            admin_wallet: ctx.accounts.admin_wallet.key(),
            tx_signature: ctx.accounts.user_wallet.key().to_string(), // Simplified for example
        })?;

        Ok(())
    }
}

// fn get_sol_price() -> Result<f64> {
//   Ok(100.0) // $100 per SOL
// }

// Emit deposit event
fn emit_deposit_event(event: DepositEvent) -> Result<()> {
    emit!(event);
    Ok(())
}

// Context for USDC deposit
#[derive(Accounts)]
pub struct DepositUsdc<'info> {
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_usdc_account: Account<'info, TokenAccount>,
    #[account(mut, signer)]
    pub user_wallet: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Context for SOL deposit
#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut, signer)]
    pub user_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub admin_wallet: AccountInfo<'info>,
    // CHECK: Chainlink feed account
    #[account(mut)]
    pub chainlink_feed: AccountInfo<'info>,
    // CHECK: Chainlink program account
    #[account(mut)]
    pub chainlink_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// Event structure
#[event]
pub struct DepositEvent {
    pub user_wallet: Pubkey,      // User's wallet address
    pub deposit_amount: u64,      // Amount deposited
    pub deposit_currency: String, // "USDC" or "SOL"
    pub usd_value: f64,           // USD value of deposit
    pub bvc_amount: u64,          // Equivalent BVC amount (10 BVC per 1 USD)
    pub admin_wallet: Pubkey,     // Admin wallet that received funds
    pub tx_signature: String,     // Transaction signature for reference
}
