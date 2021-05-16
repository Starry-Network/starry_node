# Sub Module

Create Sub tokens from NFT.

### Terminology

* **Sub Token:** Lock NFT to this module then create new collection and tokens.
* **Recover:** Restore Sub Token to NFT.

## Interface

### Dispatchable Functions

* `create` - Transfer NFT to this module, and then create a new collection.
* `recover` - Transfer the locked NFT to the account that has all the sub tokens, and destroy the sub tokens.
* `mint_non_fungible` -  Mint one or a batch of SubNFTs 
* `mint_fungible` - Mint some SubFTs

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html
