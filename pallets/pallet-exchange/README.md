# Exchange Module

Exchange NFTs or FTs.

For pool, use bancor curve.
y = m * x ^ n
r = reverseRatio  = ppm / 1000000
after integral and simplify,
can get these formula
buy: p =  poolBalance * ((1 + amount / totalSupply) ** (1 / (reserveRatio)) - 1)
sell: p = poolBalance * ( 1 - ( 1 - amount / totalSupply ) ** (1 / reserveRatio))
current price = poolBalance / (totalSupply * reserveRatio)
when supply is 0, p = reserveRatio * m * amount ** (1/reserveRatio)
Thanks for the explanation in Slava Balasanov's article (https://blog.relevant.community/bonding-curves-in-depth-intuition-parametrization-d3905a681e0a)

### Terminology

* **Pool:** It can be exchanged with some FTs, and the price can be automatically discovered through bancor curve.

## Interface

### Dispatchable Functions

* `sell_nft` - Sell one or a batch of NFTs.
* `buy_nft` - Buy one or a batch of NFTs.
* `cancel_nft_order` - Cancel the order and get back the NFTs locked in the pallet.
* `create_semi_token_pool` - Create a time-limited pool.
* `sell_semi_token` - Sell FTs to pool.
* `withdraw_pool` - After the time of the pool has passed, the creator of the pool can obtain the assets in the pool.

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html
