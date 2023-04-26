
## Assignment N°3 - Uniswap based DEX 

> (Polkadot Blockchain Academy - Buenos Aires 2023)


### Introduction


For this assignment, I decided to build a decentralized Uniswap v1 based exchange. Making use of this DEX, users have the possibility of `creating exchange pools` of type **Asset-Currency**, `adding liquidity` to an existing pool to get future rewards, `removing liquidity` from a pool they are liquidity providers, or just `swapping currency` to assets (or assets to currency and to other assets) as they like.


### Main terminology


* **Pool:** A pair of type Asset/Currency available for users to swap them. A pool always satisfies the function `asset_amount * currency_amount = k_constant`.


* **Asset:** A fungible token that represents both normal assets (like ERC20) and liquidity assets (liquidity proportion that a provider has over one pool).


* **Currency:** The main token of the chain.


* **Liquidity provider:** A user provider of both currency and asset amount to a new or existing pool.


* **Liquidity asset:** A fungible asset used to represent the proportion of currency and asset amount of a pool that corresponds to a provider.


### Rewards and fees in this dex project


* **Fees:** when a swap is made within a certain pool, the user pays a fee of 0.3% over the equivalent currency amount he wants to swap.


* **Reward system:** when a pool is first created, the user that created the pool receives the same amount of liquidity assets as the amount of currency that he provides. Then, when other users starts swapping in that pool, fees wiil acumulate, and so when a provider wants to remove liquidity, he will have more currency represented by the same amount of liquidity assets that he got when he created the pool.


### Pallet configuration


#### General types


* **AccountIdOf<T>:** used to represent account ids (for example the `pallet's account_id`).


* **BalanceOf<T>:** used to represent both `currency` and `asset` balances. Unifying these two types is very useful to handle only one balance type and avoid mistakes while working with currency or assets.


* **AssetIdOf<T>:** used to represent the id of a specific `fungible` asset.

#### Storage
This project makes use of the storage by saving pools within it. The structure used to save pools information is a `StorageMap` that uses the `asset_id` of a fungible asset as the key, and the data associated to that key as a `pool struct` that contains all the data related to a certain pool.


```rust
pub struct Pool<AssetId, AssetBalance> {
		pub asset_id: AssetId,
		pub currency_reserve: AssetBalance,
		pub asset_reserve: AssetBalance,
		pub liquidity_asset_id: AssetId,
	}
```


One asset_id can **only** have **one** pool associated to it.


>**Note:** `AssetId` and `AssetBalance` are types defined within the pallet config itself. They are equivalent to `AssetIdOf<T>` and `BalanceOf<T>`. 

### Genesis configuration
In the GenesisConfig, only for `--dev` mode, four assets are created with asset_ids of `[1, 2, 3, 4]`, each one with **1000**`u128` initial amount. The pallet account is also initializated with **1000**`u128` amount of currency.

### Extrinsics


#### create_pool()


Allow users to create a pool. The creator of a pool must indicate the amount of both currency and asset to put into the pool. He then will receive as many liquidity assets as the currency amount indicated. The user also has to indicate the id of the liquidity asset to create.


##### Parameters
* **origin:** Caller´s acount id. The call must be signed.


* **asset_id:** Id of the fungible asset to associate to the new pool.


* **liquidity_asset_id:** Id of the fungible asset to create as the liquidity asset of the new pool.


* **currency_amount:** Currency amount to spend.


* **asset_amount:** Asset amount to spend of the **asset_id** indicated previously.

##### Events


* **PoolCreated:** event that indicates the pool was created successfully.


* **LiquidityAdded:** event that indicates that liquidity was provided successfully to the new pool.


##### Errors


* **CurrencyAmountZero:** the `currency_amount` indicated is zero.


* **AssetAlreadyExists:** `liquidity_asset_id` already exists.


* **AssetNotFound:** `asset_id` not found.


* **PoolAlreadyExists:** there is already one pool created with the requested `asset_id`.


* **AssetAmountZero:** the `asset_amount` indicated is zero.

#### add_liquidity()


Allows users to be liquidity providers of one existing pool. The caller must indicate the currency amount he want to spend, and then the amount of fungibles assets to insert into the pool and the liquidity assets to mint to the user are calculated by the following functions: 


* **asset_amount:** `((currency_amount/ currency_pool_reserve) * asset_pool_reserve) + 1`


* **liquidity_asset_amount:** `(currency_amount/ currency_pool_reserve) * liquidity_asset_pool_reserve`

##### Parameters


* **origin:** Caller´s acount id. The call must be signed.


* **asset_id:** Id of the fungible asset to associate to the pool.


* **currency_amount:** Currency amount to spend.

##### Events


* **LiquidityAdded:** event that indicates that liquidity was provided successfully to the existing pool.


##### Errors


* **CurrencyAmountZero:** the `currency_amount` indicated is zero.


* **PoolNotFund:** a pool associated to the requested `asset_id` was not found.


* **OperationOverflow:** one of the math calculations resulted in an overflow.

#### remove_liquidity()


Allows users to remove liquidity from an existing pool they are providers of. The caller must indicate the pool which he wants to extract funds of and the liquidity assets amount to burn. Then, the currency amount and fungible asset amount to withdraw are calculated by the following functions: 

* **currency_amount:** `(liquidity_amount/asset_liq_total_issuance) * currency_reserve`


* **asset_amount:** `(liquidity_amount/asset_liq_total_issuance) * asset_reserve`

##### Parameters


* **origin:** Caller´s acount id. The call must be signed.


* **asset_id:** Id of the fungible asset to associate to the pool.


* **liquidity_amount:** Liquidity assets amount to burn.


##### Events


* **LiquidityRemoved:** event that indicates that liquidity was removed successfully from the existing pool.


##### Errors
* **LiqAmountZero:** the `liquidity_amount` indicated is zero.


* **PoolNotFund:** a pool associated to the requested `asset_id` was not found.

#### currency_to_asset()


Allows users to swap an amount of currency for an amount of a fungible asset. This asset must have a pool associated for the swap to execute.


##### Parameters


* **origin:** Caller´s acount id. The call must be signed.


* **currency_amount:** currency amount to swap.


* **asset_id:** Id of the fungible asset to swap for.


##### Events


* **CurrencyToAsset:** event that indicates the swap was executed successfully.


##### Errors
* **CurrencyAmountZero:** the `currency_amount` indicated is zero.


* **AssetNotFound:** `asset_id` not found.


* **PoolNotFund:** a pool associated to the requested `asset_id` was not found.


* **OperationOverflow:** one of the math calculations resulted in an overflow.


#### asset_to_currency()


Allows users to swap an amount of a fungible asset for an amount of currency. The asset must have a pool associated for the swap to execute.


##### Parameters


* **origin:** Caller´s acount id. The call must be signed.


* **asset_amount:** asset amount to swap.


* **asset_id:** Id of the fungible asset to swap from.

##### Events


* **AssetToCurrency:** event that indicates the swap was executed successfully.

##### Errors


* **AssetAmountZero:** the `asset_amount` indicated is zero.


* **AssetNotFound:** `asset_id` not found.


* **PoolNotFund:** a pool associated to the requested `asset_id` was not found.


* **OperationOverflow:** one of the math calculations resulted in an overflow.


#### asset_to_asset()
Allows users to swap an amount of a fungible asset for an amount of another fungible asset. Both assets must have a pool associated for the swap to execute.


##### Parameters
* **origin:** Caller´s acount id. The call must be signed.


* **asset_id_from:** Id of the fungible asset to swap from.


* **asset_id_to:** Id of the fungible asset to swap to.


* **asset_amount:** asset amount to swap.

##### Events
* **AssetToAsset:** event that indicates the swap was executed successfully.


##### Errors
* **AssetAmountZero:** the `asset_amount` indicated is zero.


* **AssetNotFound:** `asset_id` not found.


* **PoolNotFund:** a pool associated to the requested `asset_id` was not found.


* **OperationOverflow:** one of the math calculations resulted in an overflow.


#### mint_asset() 
>**`Only for testing purpose, should be removed in production`**

Allows a user to mint an amount of an specific `asset_id`. This extrinsic is used to have some assets to interact with PolkadotJs without modifying the GenesisConfig for users to have balance. This extrinsic should be removed in production and then manage the asset minting in a better way.

##### Parameters
* **origin:** Caller´s acount id. The call must be signed.


* **asset_id:** Id of the fungible asset to mint.


* **asset_amount:** asset amount to mint.

### API Price Oracle
Within the pallet, there is a **public** function called `price_oracle` that receives an `asset_id` as a parameter. This function calculate the common minimum between both reserves (currency and asset) of the pool associated to the `asset_id` indicated. Then, it divides each reserve amount by the minimum calculated previously, and returns the pair `(asset_amount, currency_amount)`. With this pair, is possible to see wich is the price comparing both quantities, for example (1 ETH/ 300 DOT).

This function returns a Result<> with an instance of the following struct:

```rust
pub struct OraclePrice<AssetId, AssetBalance> {
		pub asset_id: AssetId,
		pub asset_amount: AssetBalance,
		pub currency_amount: AssetBalance,
	}
```

### Testing
To run all the test suites, make a `cd` to `/pba-assignment-3-Agusrodri/substrate-node-template` and run the command `cargo test -p pallet-dex`.

### Running the node and interacting with PolkadotJs
To run the node:
* First make a `cd` to `/pba-assignment-3-Agusrodri/substrate-node-template`


* Run `cargo check -p node-template-runtime --release`


* If there are no errors, run `cargo build --release`


* When the build finishes, run `./target/release/node-template --dev`


* After these steps, open PolkadotJs on your favourite browser and start swapping! 


>**Note:** Before you start creating pools and swapping, ensure you minted some assets to your account with the `mint_asset()` extrinsic. Otherwise you will not be able to make any operation with the node.

### Future improvements
Personally, I really enjoyed developing this project. I think I could improve it by adding v2 Uniswap features to it, like providing the possibility of creating pools of type **Asset/Asset**. Also, I think that I could modularize the code for each part to be more reusable.


















