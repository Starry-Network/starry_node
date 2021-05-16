# Graph Module

Combine different tokens together.

### Terminology

* **Link:** Link a token to another token and the linked token must be NFT.
* **Parent NFT:** Linked NFT.
* **Child Token:** Link to the parent's token.
* **Ancestor NFT:** NFT located before parent NFT or parent NFT.
* **Root NFT:** Graph token's starting NFT.

## Interface

### Dispatchable Functions

* `link_non_fungible` - Link a NFT to another NFT.
* `link_fungible` - Link some FTs to NFT.
* `recover_non_fungible` - Transfer a child NFT to root_owner.
* `recover_fungible` - Transfer some child FTs to root_owner.
* `burn_fungible` - Destroy some FTs by owner
* `burn_non_fungible` - Destroy one or a batch of NFTs NFTs by owner

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html
