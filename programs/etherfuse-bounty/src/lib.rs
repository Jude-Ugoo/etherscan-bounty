use anchor_lang::prelude::*;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
    Metadata as Metaplex,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, mint_to, Burn, Mint, MintTo, Token, TokenAccount},
};

declare_id!("rtGDFZ7iMErBqCZYbP794g3tfsvEeWRQoWMDcRxscCM");

#[program]
pub mod etherfuse_bounty {
    use super::*;

    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        metadata: InitTokenParams,
    ) -> Result<()> {
        //? PDA seeds and bump to "sign" for CPI
        let seeds = b"stablecoin_mint";
        let bump = ctx.bumps.token_mint;
        let signer: &[&[&[u8]]] = &[&[seeds, &[bump]]];

        //? On-chain token metadata for the mint
        let token_data = DataV2 {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        //? CPI Context
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                //? the metadata account being created
                metadata: ctx.accounts.metadata.to_account_info(),
                //? the mint account of the metadata account
                mint: ctx.accounts.token_mint.to_account_info(),
                //? the mint authority of the mint account
                mint_authority: ctx.accounts.token_mint.to_account_info(),
                //? the update authority of the metadata account
                update_authority: ctx.accounts.token_mint.to_account_info(),
                //? the payer for creating the metadata
                payer: ctx.accounts.user.to_account_info(),
                //? the system program account
                system_program: ctx.accounts.system_program.to_account_info(),
                //? the rent sysvar account
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer,
        );

        create_metadata_accounts_v3(
            cpi_ctx,    //? cpi context
            token_data, //? token metadata
            false,       //? is_mutable
            true,       //? update_authority_is_signer
            None,       //? collection_details
        )?;

        msg!("Token mint created successfully By Country Man");

        Ok(())
    }

    pub fn mint_stablecoin(ctx: Context<MintStableCoin>, quantity: u64) -> Result<()> {
        //? PDA seeds and bump to "sign" for CPI
        let seeds = b"stablecoin_mint";
        let bump = ctx.bumps.token_mint;
        let signer: &[&[&[u8]]] = &[&[seeds, &[bump]]];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.token_mint.to_account_info(),
                },
                signer,
            ),
                quantity,
        )?;

        Ok(())
    }

    pub fn burn_token(ctx: Context<BurnToken>, quantity: u64) -> Result<()> {

        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    from: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            quantity,
        )?;

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(
    params: InitTokenParams
)]
pub struct InitializeToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    // The PDA is both the address of the mint account and the mint authority
    #[account(
        init,
        payer = user,
        seeds = [b"stablecoin_mint"],
        bump,
        mint::decimals = params.decimals,
        mint::authority = token_mint
    )]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    // pub token_data: Account<'info, TokenData>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintStableCoin<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stablecoin_mint"],
        bump,
        mint::authority = token_mint
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stablecoin_mint"],
        bump,
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}