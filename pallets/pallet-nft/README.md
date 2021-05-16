# NFT Module

Create a batch of NonFungible or Fungible Tokens.

### Terminology

* **Mint:** Mint one or a batch of NFTs or some FTs (SemiFts)
* **transfer:** Transfer one or a batch of tokens from one account to another account
* **Burn:** Destroy one or a batch of tokens from an account. This is an irreversible operation.
* **Fungible Token:** Fungible or semi-fungible token
* **Non-fungible asset:** Unique or have some copies of the token.

## Interface

### Dispatchable Functions

* `mint_fungible` - Mint some FTs
* `mint_non_fungible` - Mint one or a batch of NFTs
* `transfer_fungible` - Transfer some FTs to another account
* `transfer_non_fungible` - Transfer one or a batch of NFTs to another account
* `burn_fungible` - Destroy some FTs by owner
* `burn_non_fungible` - Destroy one or a batch of NFTs NFTs by owner

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html
