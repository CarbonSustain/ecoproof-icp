#!/bin/bash

DEPLOY_NETWORK=${1:-local}
echo "📡 Deploying to network: $DEPLOY_NETWORK"

# Block mainnet for safety
if [ "$DEPLOY_NETWORK" = "ic" ]; then
  echo "❌ You are trying to deploy to mainnet. Please use playground or local only."
  exit 1
fi

# 🖥️ Local: run replica, deploy everything
if [ "$DEPLOY_NETWORK" = "local" ]; then
  echo "🔁 Stopping any running local replica..."
  dfx stop
  echo "🚀 Starting clean local replica..."
  dfx start --clean --background

  echo "📦 Creating local canisters..."
  dfx canister create https_outbound_canister
  dfx canister create ecoproof-icp-backend
  dfx canister create icrc1_ledger_canister

  echo "🔧 Building all canisters..."
  dfx build

  echo "💰 Deploying ICRC-1 Ledger with Init arguments..."
  dfx deploy icrc1_ledger_canister --argument '(
    variant { Init = record {
      token_symbol = "CST";
      token_name = "CarbonSustain Token";
      minting_account = record { owner = principal "zqysp-sinpb-fpwv7-tenyi-qfi3z-7jfuh-dlbnc-lwrbk-vckvr-mdwpv-zqe" };
      transfer_fee = 1000;
      metadata = vec {};
      feature_flags = opt record { icrc2 = true };
      initial_balances = vec {
        record {
          record { owner = principal "zebce-iomlc-nkbqr-wgnia-dtbv2-2olwr-qbxfa-6intj-pcw72-it3xt-jae" }; 100_000_000_000
        }
      };
      archive_options = record {
        num_blocks_to_archive = 1000;
        trigger_threshold = 2000;
        controller_id = principal "pdvbi-eykrt-uflfw-oi7h6-mixap-t3ac6-bntie-rskcw-wa2zg-fdhgf-5ae";
        cycles_for_archive_creation = opt 10_000_000_000_000
      }
    }}
  )'

  echo "📂 Re-deploying ICRC-1 Ledger from source dir..."
  cd src/icrc1_ledger_canister
  dfx deploy icrc1_ledger_canister --argument '(
    variant { Init = record {
      token_symbol = "CST";
      token_name = "CarbonSustain Token";
      minting_account = record { owner = principal "zqysp-sinpb-fpwv7-tenyi-qfi3z-7jfuh-dlbnc-lwrbk-vckvr-mdwpv-zqe" };
      transfer_fee = 1000;
      metadata = vec {};
      feature_flags = opt record { icrc2 = true };
      initial_balances = vec {
        record {
          record { owner = principal "zebce-iomlc-nkbqr-wgnia-dtbv2-2olwr-qbxfa-6intj-pcw72-it3xt-jae" }; 100_000_000_000
        }
      };
      archive_options = record {
        num_blocks_to_archive = 1000;
        trigger_threshold = 2000;
        controller_id = principal "pdvbi-eykrt-uflfw-oi7h6-mixap-t3ac6-bntie-rskcw-wa2zg-fdhgf-5ae";
        cycles_for_archive_creation = opt 10_000_000_000_000
      }
    }}
  )'
  cd ../..

  echo "🚀 Deploying local canisters..."
  dfx deploy https_outbound_canister
  dfx deploy ecoproof-icp-backend

  echo "🚀 Deploying dao_backend locally..."
  dfx deploy dao_backend
fi

# 🌐 Playground: only deploy dao_backend
if [ "$DEPLOY_NETWORK" = "playground" ]; then
  echo "🔧 Building dao_backend..."
  dfx build dao_backend

  echo "🚀 Deploying dao_backend to Playground..."
  dfx deploy dao_backend --playground

  echo "🌍 Playground URL:"
  echo "https://$(dfx canister id dao_backend --playground).icp0.io"
fi

echo "✅ Deployment completed to $DEPLOY_NETWORK."
