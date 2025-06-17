#!/bin/bash

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
    echo "🔑 Loaded environment variables from .env file"
else
    echo "❌ .env file not found! Please create one with WEATHER_API_KEY=your_api_key"
    exit 1
fi

# Check if WEATHER_API_KEY is set
if [ -z "$WEATHER_API_KEY" ]; then
    echo "❌ WEATHER_API_KEY not found in .env file!"
    echo "Please add: WEATHER_API_KEY=your_openweather_api_key_here"
    exit 1
fi

echo "🔁 1. Stop any running replica and start a clean local replica..."
dfx stop
dfx start --clean --background

echo "📦 2. Create required canisters..."
dfx canister create https_outbound_canister
dfx canister create ecoproof-icp-backend
dfx canister create --all

echo "🔧 3. Build all canisters..."
dfx build

echo "🌤️ 4. Deploy HTTPS Outbound Canister with API key..."
dfx deploy https_outbound_canister --argument "(opt record { openweather_api_key = \"$WEATHER_API_KEY\" })"

echo "💰 5. Deploy ICRC-1 Ledger canister locally with Init arguments..."
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

echo "📂 6. Re-deploying ICRC-1 Ledger from within its directory (if needed)..."
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

echo "🚀 7. Deploy remaining canisters locally..."
dfx deploy dao_backend
dfx deploy ecoproof-icp-backend

echo "✅ All canisters deployed locally. Done!"
echo "🌤️ HTTPS Outbound Canister deployed with API key from .env file"
