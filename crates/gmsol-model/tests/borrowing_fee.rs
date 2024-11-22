use std::time::Duration;

use gmsol_model::{
    fixed::Fixed,
    num::MulDiv,
    params::{fee::BorrowingFeeParams, PriceImpactParams},
    price::Prices,
    test::{TestMarket, TestMarketConfig, TestPosition},
    Balance, BaseMarketExt, BorrowingFeeMarket, BorrowingFeeMarketExt, LiquidityMarketMutExt,
    PositionMutExt,
};

#[test]
fn test_total_borrowing_with_high_borrowing_factor() -> gmsol_model::Result<()> {
    let unit = Fixed::<u128, 20>::ONE.into_inner();
    let max_open_interest = 1_157_920_900_000 * unit;
    let amount = 5789604500 * 10_u128.pow(9);

    let mut market = TestMarket::<u128, 20>::with_config(TestMarketConfig {
        borrowing_fee_params: BorrowingFeeParams::builder()
            .receiver_factor(0)
            .exponent_for_long(unit)
            .exponent_for_short(unit)
            .factor_for_long(2_780_000_000_000 * 20)
            .factor_for_short(2_780_000_000_000 * 20)
            .build(),
        max_open_interest,
        swap_impact_params: PriceImpactParams::builder()
            .positive_factor(0)
            .negative_factor(0)
            .exponent(unit)
            .build(),
        position_impact_params: PriceImpactParams::builder()
            .positive_factor(0)
            .negative_factor(0)
            .exponent(unit)
            .build(),
        min_collateral_factor_for_oi: 0,
        ..Default::default()
    });

    let prices = Prices::new_for_test(20_000_000_000_000, 20_000_000_000_000, 100_000_000_000_000);

    market.deposit(amount, amount, prices)?.execute()?;

    let mut position = TestPosition::long(true);
    _ = position
        .ops(&mut market)
        .increase(prices, amount * 1_000_000, max_open_interest, None)?
        .execute()?;

    let factor = market.borrowing_factor_per_second(true, &prices)?;
    println!("borrowing factor per second: {factor}");

    market.move_clock_forward(Duration::from_secs(3600 * 24 * 365 * 100));

    _ = position
        .ops(&mut market)
        .decrease(prices, 0, None, 0, false, false)?
        .execute()?;

    let total_borrowing = market.total_borrowing_pool()?;
    println!("{total_borrowing:#?}");

    Ok(())
}

#[test]
fn test_total_borrowing_with_high_borrowing_factor_2() -> gmsol_model::Result<()> {
    let unit = Fixed::<u128, 20>::ONE.into_inner();

    let price_decimals = 10_u128.pow(9);
    let unit_token = 10_u128.pow(11);
    let price = 100_000 * price_decimals;
    let amount = i128::MAX as u128 / unit_token / price;
    let oi = amount * unit_token * price;
    let factor_per_second = 100_000_000_000 / 100 * unit / (365 * 24 * 60 * 60);
    // borrowing_factor_per_second == factor * (oi * unit / pool_value) / unit == factor * oi / pool_value
    let factor = factor_per_second
        .checked_mul_div(&(amount * unit_token * price), &oi)
        .unwrap();

    let mut market = TestMarket::<u128, 20>::with_config(TestMarketConfig {
        max_pool_amount: u128::MAX,
        max_open_interest: u128::MAX,
        max_pool_value_for_deposit: u128::MAX,
        borrowing_fee_params: BorrowingFeeParams::builder()
            .receiver_factor(0)
            .exponent_for_long(unit)
            .exponent_for_short(unit)
            .factor_for_long(factor)
            .factor_for_short(factor)
            .build(),
        swap_impact_params: PriceImpactParams::builder()
            .positive_factor(0)
            .negative_factor(0)
            .exponent(unit)
            .build(),
        position_impact_params: PriceImpactParams::builder()
            .positive_factor(0)
            .negative_factor(0)
            .exponent(unit)
            .build(),
        min_collateral_factor_for_oi: 0,
        ..Default::default()
    });

    let prices = Prices::new_for_test(price, price, price);

    market
        .deposit(amount * unit_token, amount * unit_token, prices)?
        .execute()?;

    let mut position = TestPosition::long(true);
    _ = position
        .ops(&mut market)
        .increase(prices, amount * unit_token, oi, None)?
        .execute()?;

    let factor = market.borrowing_factor_per_second(true, &prices)?;
    println!("borrowing factor per second: {factor}");

    market.move_clock_forward(Duration::from_secs(3600 * 24 * 365 * 100));

    let mut position_2 = TestPosition::long(true);
    _ = position_2
        .ops(&mut market)
        .increase(prices, 100 * unit_token, 100 * unit_token * price, None)?
        .execute()?;

    let open_interest = market.open_interest()?.long_amount()?;
    let total_borrowing = market.total_borrowing_pool()?;
    let cbf = market.cumulative_borrowing_factor(true)?;
    println!("open interest: {open_interest:#?}");
    println!("total borrowing: {total_borrowing:#?}");
    println!("cumulative borrowing factor: {cbf:#?}");
    Ok(())
}
