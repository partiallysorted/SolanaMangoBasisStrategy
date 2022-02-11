use std::num::NonZeroU64;

use anchor_lang::{
    prelude::{AccountInfo, ProgramError, ProgramResult},
    Key, ToAccountMetas,
};
use az::Cast;
use fixed::types::I80F48;
use mango::{
    instruction::{
        consume_events, create_mango_account, create_spot_open_orders, deposit, place_perp_order,
        withdraw, MangoInstruction,
    },
    matching::{OrderType, Side as MangoSide},
    state::MangoCache,
};
use mango_common::Loadable;
use serum_dex::{
    instruction::{NewOrderInstructionV3, SelfTradeBehavior},
    matching::Side as SerumSide,
};
use solana_program::{
    instruction::Instruction,
    program::{invoke, invoke_signed},
};

pub fn create_account<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    account_num: u64,
) -> ProgramResult {
    let instruction = create_mango_account(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &owner.key(),
        &system_program.key(),
        &payer.key(),
        account_num,
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            owner.to_owned(),
            system_program.to_owned(),
            payer.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn create_open_orders<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    serum_dex: &AccountInfo<'info>,
    spot_open_orders: &AccountInfo<'info>,
    spot_market: &AccountInfo<'info>,
    mango_signer: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let instruction = create_spot_open_orders(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &owner.key(),
        &serum_dex.key(),
        &spot_open_orders.key(),
        &spot_market.key(),
        &mango_signer.key(),
        &payer.key(),
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            owner.to_owned(),
            serum_dex.to_owned(),
            spot_open_orders.to_owned(),
            spot_market.to_owned(),
            mango_signer.to_owned(),
            system_program.to_owned(),
            payer.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn deposit_usdc<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_root_bank: &AccountInfo<'info>,
    mango_node_bank: &AccountInfo<'info>,
    mango_vault: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    amount: u64,
) -> ProgramResult {
    let instruction = deposit(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &authority.key(),
        &mango_cache.key(),
        &mango_root_bank.key(),
        &mango_node_bank.key(),
        &mango_vault.key(),
        &token_account.key(),
        amount,
    )
    .expect("deposit instruction failed");
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            authority.to_owned(),
            mango_cache.to_owned(),
            mango_root_bank.to_owned(),
            mango_node_bank.to_owned(),
            mango_vault.to_owned(),
            token_program.to_owned(),
            token_account.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn withdraw_usdc<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_root_bank: &AccountInfo<'info>,
    mango_node_bank: &AccountInfo<'info>,
    mango_vault: &AccountInfo<'info>,
    mango_signer: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    amount: u64,
) -> ProgramResult {
    let mango_spot_open_orders = ["11111111111111111111111111111111".parse().unwrap(); 15];
    let instruction = withdraw(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &authority.key(),
        &mango_cache.key(),
        &mango_root_bank.key(),
        &mango_node_bank.key(),
        &mango_vault.key(),
        &token_account.key(),
        &mango_signer.key(),
        &mango_spot_open_orders,
        amount,
        false,
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            authority.to_owned(),
            mango_cache.to_owned(),
            mango_root_bank.to_owned(),
            mango_node_bank.to_owned(),
            mango_vault.to_owned(),
            token_account.to_owned(),
            mango_signer.to_owned(),
            authority.to_owned(),
            token_program.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn get_price<'info>(
    mango_cache: &AccountInfo<'info>,
    market_index: usize,
) -> Result<I80F48, ProgramError> {
    let cache: MangoCache = *MangoCache::load_from_bytes(&mango_cache.data.borrow())?;
    Ok(cache.get_price(market_index))
}

pub fn adjust_position_perp<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_market: &AccountInfo<'info>,
    mango_bids: &AccountInfo<'info>,
    mango_asks: &AccountInfo<'info>,
    mango_event_queue: &AccountInfo<'info>,
    spot_open_orders: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    side: MangoSide,
    amount_base: i64,
    market_index: usize,
    reduce_only: bool,
) -> ProgramResult {
    let mut mango_spot_open_orders = ["11111111111111111111111111111111".parse().unwrap(); 15];
    mango_spot_open_orders[market_index] = spot_open_orders.key();
    let instruction = place_perp_order(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &authority.key(),
        &mango_cache.key(),
        &mango_market.key(),
        &mango_bids.key(),
        &mango_asks.key(),
        &mango_event_queue.key(),
        &mango_spot_open_orders,
        side,
        i64::MAX,
        amount_base.cast(),
        1,
        OrderType::Market,
        reduce_only,
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            authority.to_owned(),
            mango_cache.to_owned(),
            mango_market.to_owned(),
            mango_bids.to_owned(),
            mango_asks.to_owned(),
            mango_event_queue.to_owned(),
            spot_open_orders.to_owned(),
        ],
        seeds,
    )?;
    let instruction = consume_events(
        &mango_program.key(),
        &mango_group.key(),
        &mango_cache.key(),
        &mango_market.key(),
        &mango_event_queue.key(),
        &mut [mango_account.key()],
        64,
    )?;
    invoke(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_cache.to_owned(),
            mango_market.to_owned(),
            mango_event_queue.to_owned(),
            mango_account.to_owned(),
        ],
    )?;
    Ok(())
}

pub fn adjust_position_spot<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_signer: &AccountInfo<'info>,
    serum_dex: &AccountInfo<'info>,
    spot_market: &AccountInfo<'info>,
    spot_bids: &AccountInfo<'info>,
    spot_asks: &AccountInfo<'info>,
    spot_request_queue: &AccountInfo<'info>,
    spot_event_queue: &AccountInfo<'info>,
    spot_base: &AccountInfo<'info>,
    spot_quote: &AccountInfo<'info>,
    spot_base_root_bank: &AccountInfo<'info>,
    spot_base_node_bank: &AccountInfo<'info>,
    spot_base_vault: &AccountInfo<'info>,
    spot_quote_root_bank: &AccountInfo<'info>,
    spot_quote_node_bank: &AccountInfo<'info>,
    spot_quote_vault: &AccountInfo<'info>,
    serum_dex_signer: &AccountInfo<'info>,
    spot_open_orders: &AccountInfo<'info>,
    srm_vault: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    side: SerumSide,
    amount: u64,
    market_lot_size: u64,
) -> ProgramResult {
    let (limit_price, max_base_quantity, max_quote_quantity) = match side {
        SerumSide::Bid => {
            let price = 10000000000 * market_lot_size;
            (
                NonZeroU64::new(price).unwrap(),
                NonZeroU64::new(amount).unwrap(),
                NonZeroU64::new(amount * price).unwrap(),
            )
        }
        SerumSide::Ask => {
            let price = 1 * market_lot_size;
            (
                NonZeroU64::new(price).unwrap(),
                NonZeroU64::new(amount).unwrap(),
                NonZeroU64::new(amount * price).unwrap(),
            )
        }
    };
    let accounts = vec![
        mango_program.to_owned(),
        //
        mango_group.to_owned(),
        mango_account.to_owned(),
        authority.to_owned(),
        mango_cache.to_owned(),
        serum_dex.to_owned(),
        spot_market.to_owned(),
        spot_bids.to_owned(),
        spot_asks.to_owned(),
        spot_request_queue.to_owned(),
        spot_event_queue.to_owned(),
        spot_base.to_owned(),
        spot_quote.to_owned(),
        spot_base_root_bank.to_owned(),
        spot_base_node_bank.to_owned(),
        spot_base_vault.to_owned(),
        spot_quote_root_bank.to_owned(),
        spot_quote_node_bank.to_owned(),
        spot_quote_vault.to_owned(),
        token_program.to_owned(),
        mango_signer.to_owned(),
        serum_dex_signer.to_owned(),
        srm_vault.to_owned(),
        spot_open_orders.to_owned(),
    ];
    let meta_accounts = accounts
        .iter()
        .skip(1) // skip program id
        .map(|x| {
            x.to_account_metas(Some(x.key() == authority.key()))
                .pop()
                .unwrap()
        })
        .collect();
    let instruction = Instruction {
        program_id: mango_program.key(),
        accounts: meta_accounts,
        data: (MangoInstruction::PlaceSpotOrder2 {
            order: NewOrderInstructionV3 {
                side,
                limit_price,
                max_coin_qty: max_base_quantity,
                max_native_pc_qty_including_fees: max_quote_quantity,
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                order_type: serum_dex::matching::OrderType::ImmediateOrCancel,
                client_order_id: 1,
                limit: 65535,
            },
        })
        .pack(),
    };
    invoke_signed(&instruction, &accounts, seeds)?;
    Ok(())
}
