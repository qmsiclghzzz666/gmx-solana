use crate::anchor_test::setup::{current_deployment, Deployment};
use gmsol_liquidity_provider as lp;
use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair, signer::Signer, system_program};

// Test helpers ----------------------------------------------------------------

fn default_gradient() -> [u128; 53] {
    // Example: linear 0.00, 0.01, 0.02, ... (scaled by 1e20). Adjust as needed in tests.
    let mut arr = [0u128; 53];
    for i in 0..53 {
        // 1% per bucket step, in 1e20 scale (i as u128 * 1e18 -> makes 0.00, 0.01, ...)
        arr[i] = (i as u128) * 10u128.pow(18);
    }
    arr
}

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
    let init_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::Initialize {
            min_stake_value: 1_000_000_000_000_000_000_000u128, // 1e21
            apy_gradient: default_gradient(),
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
    assert_eq!(gs.apy_gradient, default_gradient());
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

    // Test 3: Update APY gradient
    let mut new_grad = [0u128; 53];
    for v in new_grad.iter_mut() {
        *v = 2_000_000_000_000_000_000u128;
    }

    let update_ix = client
        .store_transaction()
        .program(lp::id())
        .anchor_args(lp::instruction::UpdateApyGradient {
            new_apy_gradient: new_grad,
        })
        .anchor_accounts(lp::accounts::UpdateApyGradient {
            global_state,
            authority: client.payer(),
        });

    let signature = update_ix.send().await?;
    tracing::info!(%signature, "updated APY gradient");

    let gs = client
        .account::<lp::GlobalState>(&global_state)
        .await?
        .expect("global_state must exist");
    assert_eq!(gs.apy_gradient, new_grad);
    tracing::info!("✓ Update APY gradient test passed");

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

    tracing::info!("🎉 All liquidity provider tests passed!");
    Ok(())
}
