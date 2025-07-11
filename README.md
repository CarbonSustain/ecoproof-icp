# `ecoproof-icp`

- cargo clean
- cargo update

- pkill -f dfx && dfx stop

- add WEATHER_API_KEY="<APIKEY>" in .env at **LINE 1 of FILE**

---
1. **Run Full Deployment Script**

   Instead of copying and pasting individual commands, simply run the deployment script:

   ```bash
   chmod +x deploy.sh
   ./deploy.sh
   ```
  # ONE COMMAND
  # cargo clean && cargo update && pkill -f dfx && dfx stop && ./deploy.sh
  # ngrok http 4943
  # save url to aws

  # if edited Candid UI
  # dfx generate && dfx deploy
  #

    When you run the deployment script, it will:
    - **Clean and update Rust dependencies**
    - **Start a fresh local DFX replica**
    - **Create and build all canisters**
    - **Deploy the ICRC-1 Ledger canister with `Init` arguments**
    - **Deploy all remaining canisters locally**
---
2. **test_insert.sh - Demo Data Insertion Script**

    This script generates sample data for local or Playground testing of the dao_backend canister. Simply run:

   ```bash
   chmod +x test_insert.sh
   ./test_insert.sh
   ```

    When you run the deployment script, it will:
    - **Creates 10 Test Users**
      - Each user has a unique Telegram ID, name, and profile picture URL.
    - **Submits 20 Weather Reports**
      - Weather data is submitted by users across 7 different cities, covering a variety of weather conditions.
    - **Votes on First 10 Submissions**

    ### Data Summary

   | Category        | Count |
   |----------------|-------|
   | Users Created   | 10    |
   | Weather Posts   | 20    |
   | Votes Cast      | 100   |
---
3. **Optional: If You Encounter Port Issues**

   If you see an error like this when starting the local replica: Failed to bind socket to 127.0.0.1:4943

    It usually means a previous replica is still running.  
    You can reset and free the port by running:

    ```bash
    pkill -f dfx
    dfx stop
    ```
    Then, restart the deployment script:

   ```bash
   ./deploy.sh
   ```
---
4. **Run Playground Deployment Script**
   Instead of copying and pasting individual commands, simply run the deployment script:

   ```bash
   chmod +x deploy_playground.sh
   ./deploy_playground.sh
   ```
    When you run the deployment script, it will:
    - **Build the dao_backend canister locally**
    - **Deploy the dao_backend to the Playground**
    - **Provide an external Playground URL you can access publicly**

   After successful deployment, you will see an output like:
   ```bash
   Playground URL:
   https://<canister_id>.icp0.io
   Deployment completed to playground.
   ```
   You can open this URL in any browser to interact with the deployed canister.

---
- If you want to check how the backend function works, go to this link: http://127.0.0.1:4943/?canisterId=bw4dl-smaaa-aaaaa-qaacq-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai
- dfx canister call ecoproof-icp-backend fetch_https '( "https://api.exchange.coinbase.com/products/ICP-USD/ticker" )'

Welcome to your new `ecoproof-icp` project and to the Internet Computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with `ecoproof-icp`, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/setup/deploy-locally)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd ecoproof-icp/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `ic` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
