use crate::anchor_test::setup::{current_deployment, Deployment};
use gmsol_liquidity_provider as lp;
use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair, signer::Signer, system_program};

// Test helpers ----------------------------------------------------------------

fn derive_global_state() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[lp::GLOBAL_STATE_SEED], &lp::id())
}

// Tests -----------------------------------------------------------------------

#[tokio::test]
async fn liquidity_provider_tests() -> eyre::Result<()> {
    let deployment = current_deployment().await?;
    let _guard = deployment.use_accounts().await?;
    let span = tracing::info_span!("liquidity_provider_tests");
    let _enter = span.enter();

    let client = deployment.user_client(Deployment::DEFAULT_USER)?;
    let (global_state, _bump) = derive_global_state();
    let gt_mint = Keypair::new();

    // Initialize the GlobalState once for all tests
    let initial_apy: u128 = 1_000_000_000_000_000_000u128; // 1% (1e20-scaled)

    let init_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::Initialize {
            min_stake_value: 1_000_000_000_000_000_000_000u128, // 1e21
            initial_apy,
        })
        .anchor_accounts(lp::accounts::Initialize {
            global_state,
            authority: client.payer(),
            gt_mint: gt_mint.pubkey(),
            system_program: system_program::ID,
        });

    let signature = init_ix.send().await?;
    tracing::info!(%signature, "initialized liquidity provider program for all tests");

    // Test 1: Verify initialization
    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");

    assert_eq!(gs.authority, client.payer());
    assert_eq!(gs.gt_mint, gt_mint.pubkey());
    assert_eq!(gs.min_stake_value, 1_000_000_000_000_000_000_000u128);

    // Verify all buckets have the same initial APY
    let expected_apy = 1_000_000_000_000_000_000u128;
    for (i, &apy) in gs.apy_gradient.iter().enumerate() {
        assert_eq!(
            apy, expected_apy,
            "Bucket {} should have APY {}",
            i, expected_apy
        );
    }
    tracing::info!("✓ Initialization test passed");

    // Test 2: Update min stake value
    let new_min: u128 = 5_000_000_000_000_000_000_000u128; // 5e21
    let update_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::UpdateMinStakeValue {
            new_min_stake_value: new_min,
        })
        .anchor_accounts(lp::accounts::UpdateMinStakeValue {
            global_state,
            authority: client.payer(),
        });

    let signature = update_ix.send().await?;
    tracing::info!(%signature, "updated min stake value");

    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");
    assert_eq!(gs.min_stake_value, new_min);
    tracing::info!("✓ Update min stake value test passed");

    // Test 3: Update APY gradient over full range using range updater
    let mut new_grad = [0u128; 53];
    for v in new_grad.iter_mut() {
        *v = 2_000_000_000_000_000_000u128;
    }

    let update_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::UpdateApyGradientRange {
            start_bucket: 0u8,
            end_bucket: 52u8,
            apy_values: new_grad.to_vec(),
        })
        .anchor_accounts(lp::accounts::UpdateApyGradient {
            global_state,
            authority: client.payer(),
        });

    let signature = update_ix.send().await?;
    tracing::info!(%signature, "updated APY gradient (full range)");

    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");
    assert_eq!(gs.apy_gradient, new_grad);
    tracing::info!("✓ Update APY gradient (full range) test passed");

    // Test 3.5: Test sparse APY gradient update (Vec-based)
    let bucket_indices: Vec<u8> = vec![0, 10, 25, 52];
    let apy_values: Vec<u128> = vec![
        5_000_000_000_000_000_000u128,  // 5%
        7_000_000_000_000_000_000u128,  // 7%
        3_000_000_000_000_000_000u128,  // 3%
        10_000_000_000_000_000_000u128, // 10%
    ];

    let sparse_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::UpdateApyGradientSparse {
            bucket_indices: bucket_indices.clone(),
            apy_values: apy_values.clone(),
        })
        .anchor_accounts(lp::accounts::UpdateApyGradient {
            global_state,
            authority: client.payer(),
        });

    let signature = sparse_ix.send().await?;
    tracing::info!(%signature, "updated sparse APY gradient");

    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");

    // Verify sparse updates were applied correctly
    for (i, &bucket_idx) in bucket_indices.iter().enumerate() {
        let expected_apy = apy_values[i];
        assert_eq!(
            gs.apy_gradient[bucket_idx as usize], expected_apy,
            "Bucket {} should have APY {}",
            bucket_idx, expected_apy
        );
    }
    tracing::info!("✓ Sparse APY gradient update test passed");

    // Test 3.6: Test range APY gradient update
    let range_start = 5u8;
    let range_end = 15u8;
    let range_values = vec![
        6_000_000_000_000_000_000u128,  // Bucket 5: 6%
        6_500_000_000_000_000_000u128,  // Bucket 6: 6.5%
        7_000_000_000_000_000_000u128,  // Bucket 7: 7%
        7_500_000_000_000_000_000u128,  // Bucket 8: 7.5%
        8_000_000_000_000_000_000u128,  // Bucket 9: 8%
        8_500_000_000_000_000_000u128,  // Bucket 10: 8.5%
        9_000_000_000_000_000_000u128,  // Bucket 11: 9%
        9_500_000_000_000_000_000u128,  // Bucket 12: 9.5%
        10_000_000_000_000_000_000u128, // Bucket 13: 10%
        10_500_000_000_000_000_000u128, // Bucket 14: 10.5%
        11_000_000_000_000_000_000u128, // Bucket 15: 11%
    ];

    let range_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::UpdateApyGradientRange {
            start_bucket: range_start,
            end_bucket: range_end,
            apy_values: range_values.clone(),
        })
        .anchor_accounts(lp::accounts::UpdateApyGradient {
            global_state,
            authority: client.payer(),
        });

    let signature = range_ix.send().await?;
    tracing::info!(%signature, "updated range APY gradient");

    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");

    // Verify range updates were applied correctly
    for (i, expected_apy) in range_values.iter().enumerate() {
        let bucket_idx = range_start as usize + i;
        assert_eq!(
            gs.apy_gradient[bucket_idx], *expected_apy,
            "Bucket {} should have APY {}",
            bucket_idx, expected_apy
        );
    }
    tracing::info!("✓ Range APY gradient update test passed");

    // Test 4: Transfer and accept authority
    // Use an existing user as the new authority
    let new_auth_client = deployment.user_client(Deployment::USER_1)?;
    let new_auth = new_auth_client.payer();

    let transfer_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::TransferAuthority {
            new_authority: new_auth,
        })
        .anchor_accounts(lp::accounts::TransferAuthority {
            global_state,
            authority: client.payer(),
        });

    let signature = transfer_ix.send().await?;
    tracing::info!(%signature, "proposed authority transfer");

    // Accept the authority transfer using the new authority client
    let accept_ix = new_auth_client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::AcceptAuthority {})
        .anchor_accounts(lp::accounts::AcceptAuthority {
            global_state,
            pending_authority: new_auth,
        });

    let signature = accept_ix.send().await?;
    tracing::info!(%signature, "accepted authority transfer");

    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");
    assert_eq!(gs.authority, new_auth);
    assert_eq!(gs.pending_authority, Pubkey::default());
    tracing::info!("✓ Authority transfer test passed");

    // Test 5: Try unauthorized update (should fail)
    let wrong = Keypair::new();
    let update_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::UpdateMinStakeValue {
            new_min_stake_value: 7_000_000_000_000_000_000_000u128,
        })
        .anchor_accounts(lp::accounts::UpdateMinStakeValue {
            global_state,
            authority: wrong.pubkey(),
        });

    let res = update_ix.send().await;
    assert!(res.is_err(), "unauthorized update should fail");
    tracing::info!("✓ Unauthorized update test passed");

    // Test 6: Try transfer to default address (should fail)
    let transfer_ix = new_auth_client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::TransferAuthority {
            new_authority: Pubkey::default(),
        })
        .anchor_accounts(lp::accounts::TransferAuthority {
            global_state,
            authority: new_auth, // Use the new authority we just set
        });

    let res = transfer_ix.send().await;
    assert!(res.is_err(), "transfer to default address should fail");
    tracing::info!("✓ Reject default address test passed");

    tracing::info!("All liquidity provider tests passed!");
    Ok(())
}
