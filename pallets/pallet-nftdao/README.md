# NFTDAO Module

A pallet that refers to Moloch and can use NFT as tribute.

### Terminology

* **Pool:** It can be exchanged with some FTs, and the price can be automatically discovered through bancor curve.
* **Action:** After the proposal is passed, the operation of the dao account on the chain.
* **Proposal Queue:** Only proposals in the queue can be voted.
* **Sponsor:** In order to prevent spam proposals, a proposal must be sponsored to enter the queue.
* **Vote:** Yes or not, only members of dao can vote.
* **Grace:** You can ragequit after the voting period.
* **Ragequit:** Burn shares and exchange for corresponding assets.

## Interface

### Dispatchable Functions

* `create_dao` - Create a new DAO.
* `submit_proposal` - Submit a proposal, regardless of whether it is a member of dao can perform this operation.
* `cancel_proposal` - The proposal can be cancelled before it is sponsored.
* `sponsor_proposal` - Sponsor a proposal and make it into the queue.
* `vote_proposal` - DAO members can vote on proposals.
* `process_proposal` - After the grace period, the proposal needs to be processed.
* `ragequit` - Burn shares and exchange for corresponding assets..

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html
