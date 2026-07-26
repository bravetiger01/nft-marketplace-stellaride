#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

// ── Types ──────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct NFT {
    pub token_id: String,
    pub owner: Address,
    pub creator: Address,
    pub metadata_uri: String,
    pub collection: String,
    pub royalty_bps: u32,
    pub minted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Listing {
    pub seller: Address,
    pub price: i128,
    pub listed_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Offer {
    pub buyer: Address,
    pub amount: i128,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Collection {
    pub name: String,
    pub creator: Address,
    pub description: String,
    pub image_uri: String,
}

#[contracttype]
pub enum DataKey {
    Admin,
    FeeBps,
    PaymentToken,
    Nft(String),
    Listing(String),
    Offer(String, Address),
    Collection(String),
    UserNfts(Address),
    CollectionNfts(String),
    AllNfts,
}

// ── Contract ───────────────────────────────────────────────────────────────

#[contract]
pub struct NftMarketplace;

#[contractimpl]
impl NftMarketplace {
    // ── Admin ──

    pub fn initialize(env: Env, admin: Address, fee_bps: u32, payment_token: Address) {
        assert!(fee_bps <= 1000, "fee max 10%");
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        env.storage().instance().set(&DataKey::PaymentToken, &payment_token);
        env.storage()
            .persistent()
            .set(&DataKey::AllNfts, &Vec::<String>::new(&env));
    }

    pub fn get_fee(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0)
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap()
    }

    pub fn get_payment_token(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::PaymentToken)
            .unwrap()
    }

    pub fn set_fee(env: Env, new_fee: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        assert!(new_fee <= 1000, "fee max 10%");
        env.storage().instance().set(&DataKey::FeeBps, &new_fee);
    }

    // ── NFT Minting ──

    pub fn mint(
        env: Env,
        caller: Address,
        token_id: String,
        metadata_uri: String,
        collection: String,
        royalty_bps: u32,
    ) {
        caller.require_auth();
        assert!(royalty_bps <= 1000, "royalty max 10%");
        assert!(
            !env.storage().persistent().has(&DataKey::Nft(token_id.clone())),
            "already minted"
        );

        let nft = NFT {
            token_id: token_id.clone(),
            owner: caller.clone(),
            creator: caller.clone(),
            metadata_uri,
            collection: collection.clone(),
            royalty_bps,
            minted_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Nft(token_id.clone()), &nft);

        Self::add_to_user_nfts(&env, &caller, &token_id);

        let mut col_nfts: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::CollectionNfts(collection.clone()))
            .unwrap_or(Vec::new(&env));
        col_nfts.push_back(token_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::CollectionNfts(collection), &col_nfts);

        let mut all: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::AllNfts)
            .unwrap_or(Vec::new(&env));
        all.push_back(token_id);
        env.storage().persistent().set(&DataKey::AllNfts, &all);
    }

    pub fn get_nft(env: Env, token_id: String) -> NFT {
        env.storage()
            .persistent()
            .get(&DataKey::Nft(token_id))
            .expect("NFT not found")
    }

    // ── Transfer ──

    pub fn transfer_nft(env: Env, from: Address, to: Address, token_id: String) {
        from.require_auth();
        let mut nft: NFT = env
            .storage()
            .persistent()
            .get(&DataKey::Nft(token_id.clone()))
            .expect("NFT not found");
        assert!(nft.owner == from, "not owner");

        Self::remove_from_user_nfts(&env, &from, &token_id);
        Self::add_to_user_nfts(&env, &to, &token_id);

        nft.owner = to;
        env.storage()
            .persistent()
            .set(&DataKey::Nft(token_id.clone()), &nft);
        env.storage().persistent().remove(&DataKey::Listing(token_id));
    }

    // ── Listing ──

    pub fn list(env: Env, seller: Address, token_id: String, price: i128) {
        seller.require_auth();
        assert!(price > 0, "price must be positive");
        let nft: NFT = env
            .storage()
            .persistent()
            .get(&DataKey::Nft(token_id.clone()))
            .expect("NFT not found");
        assert!(nft.owner == seller, "not owner");
        assert!(
            !env.storage().persistent().has(&DataKey::Listing(token_id.clone())),
            "already listed"
        );

        let listing = Listing {
            seller,
            price,
            listed_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Listing(token_id), &listing);
    }

    pub fn delist(env: Env, seller: Address, token_id: String) {
        seller.require_auth();
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(token_id.clone()))
            .expect("not listed");
        assert!(listing.seller == seller, "not seller");
        env.storage()
            .persistent()
            .remove(&DataKey::Listing(token_id));
    }

    // ── Purchase ──

    pub fn buy(env: Env, buyer: Address, token_id: String) {
        buyer.require_auth();
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(token_id.clone()))
            .expect("not listed");
        assert!(buyer != listing.seller, "cannot buy own NFT");

        let mut nft: NFT = env
            .storage()
            .persistent()
            .get(&DataKey::Nft(token_id.clone()))
            .expect("NFT not found");

        let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap();
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::PaymentToken)
            .unwrap();

        let fee = listing.price * fee_bps as i128 / 10_000;
        let royalty = if nft.creator != listing.seller {
            listing.price * nft.royalty_bps as i128 / 10_000
        } else {
            0
        };
        let seller_amount = listing.price - fee - royalty;

        let token = soroban_sdk::token::Client::new(&env, &token_addr);
        if seller_amount > 0 {
            token.transfer(&buyer, &listing.seller, &seller_amount);
        }
        if fee > 0 {
            token.transfer(&buyer, &admin, &fee);
        }
        if royalty > 0 {
            token.transfer(&buyer, &nft.creator, &royalty);
        }

        Self::remove_from_user_nfts(&env, &listing.seller, &token_id);
        Self::add_to_user_nfts(&env, &buyer, &token_id);

        nft.owner = buyer;
        env.storage()
            .persistent()
            .set(&DataKey::Nft(token_id.clone()), &nft);
        env.storage().persistent().remove(&DataKey::Listing(token_id));
    }

    // ── Offers ──

    pub fn make_offer(
        env: Env,
        buyer: Address,
        token_id: String,
        amount: i128,
        expires_at: u64,
    ) {
        buyer.require_auth();
        assert!(amount > 0, "amount must be positive");
        assert!(
            expires_at > env.ledger().timestamp(),
            "expires in the past"
        );
        assert!(
            env.storage().persistent().has(&DataKey::Nft(token_id.clone())),
            "NFT not found"
        );

        let offer = Offer {
            buyer: buyer.clone(),
            amount,
            expires_at,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Offer(token_id, buyer), &offer);
    }

    pub fn accept_offer(env: Env, seller: Address, token_id: String, buyer: Address) {
        seller.require_auth();
        buyer.require_auth();
        let offer: Offer = env
            .storage()
            .persistent()
            .get(&DataKey::Offer(token_id.clone(), buyer.clone()))
            .expect("offer not found");
        assert!(
            offer.expires_at > env.ledger().timestamp(),
            "offer expired"
        );

        let mut nft: NFT = env
            .storage()
            .persistent()
            .get(&DataKey::Nft(token_id.clone()))
            .expect("NFT not found");
        assert!(nft.owner == seller, "not owner");

        let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap();
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::PaymentToken)
            .unwrap();

        let fee = offer.amount * fee_bps as i128 / 10_000;
        let royalty = if nft.creator != seller {
            offer.amount * nft.royalty_bps as i128 / 10_000
        } else {
            0
        };
        let seller_amount = offer.amount - fee - royalty;

        let token = soroban_sdk::token::Client::new(&env, &token_addr);
        if seller_amount > 0 {
            token.transfer(&buyer, &seller, &seller_amount);
        }
        if fee > 0 {
            token.transfer(&buyer, &admin, &fee);
        }
        if royalty > 0 {
            token.transfer(&buyer, &nft.creator, &royalty);
        }

        env.storage().persistent().remove(&DataKey::Offer(
            token_id.clone(),
            buyer.clone(),
        ));

        Self::remove_from_user_nfts(&env, &seller, &token_id);
        Self::add_to_user_nfts(&env, &buyer, &token_id);

        nft.owner = buyer;
        env.storage()
            .persistent()
            .set(&DataKey::Nft(token_id.clone()), &nft);
        env.storage().persistent().remove(&DataKey::Listing(token_id));
    }

    pub fn cancel_offer(env: Env, buyer: Address, token_id: String) {
        buyer.require_auth();
        assert!(
            env
                .storage()
                .persistent()
                .has(&DataKey::Offer(token_id.clone(), buyer.clone())),
            "offer not found"
        );
        env.storage()
            .persistent()
            .remove(&DataKey::Offer(token_id, buyer));
    }

    // ── Collections ──

    pub fn create_collection(
        env: Env,
        creator: Address,
        name: String,
        description: String,
        image_uri: String,
    ) {
        creator.require_auth();
        assert!(
            !env
                .storage()
                .persistent()
                .has(&DataKey::Collection(name.clone())),
            "collection exists"
        );

        let collection = Collection {
            name: name.clone(),
            creator,
            description,
            image_uri,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Collection(name.clone()), &collection);
        env.storage().persistent().set(
            &DataKey::CollectionNfts(name),
            &Vec::<String>::new(&env),
        );
    }

    pub fn get_collection(env: Env, name: String) -> Collection {
        env.storage()
            .persistent()
            .get(&DataKey::Collection(name))
            .expect("collection not found")
    }

    // ── Queries ──

    pub fn get_listings(env: Env, offset: u32, limit: u32) -> Vec<(String, Listing)> {
        let all: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::AllNfts)
            .unwrap_or(Vec::new(&env));
        let mut result = Vec::new(&env);
        let mut count = 0u32;
        let mut skipped = 0u32;

        for token_id in all.iter() {
            if let Some(listing) = env
                .storage()
                .persistent()
                .get::<_, Listing>(&DataKey::Listing(token_id.clone()))
            {
                if skipped < offset {
                    skipped += 1;
                    continue;
                }
                if count < limit {
                    result.push_back((token_id, listing));
                    count += 1;
                }
            }
        }
        result
    }

    pub fn get_user_nfts(env: Env, user: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::UserNfts(user))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_collection_nfts(env: Env, name: String) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::CollectionNfts(name))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_offer(env: Env, token_id: String, buyer: Address) -> Offer {
        env.storage()
            .persistent()
            .get(&DataKey::Offer(token_id, buyer))
            .expect("offer not found")
    }

    pub fn has_listing(env: Env, token_id: String) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Listing(token_id))
    }

    pub fn has_offer(env: Env, token_id: String, buyer: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Offer(token_id, buyer))
    }

    // ── Internal Helpers ──

    fn remove_from_user_nfts(env: &Env, user: &Address, token_id: &String) {
        let nfts: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::UserNfts(user.clone()))
            .unwrap_or(Vec::new(env));
        let mut updated = Vec::new(env);
        for id in nfts.iter() {
            if id != token_id.clone() {
                updated.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&DataKey::UserNfts(user.clone()), &updated);
    }

    fn add_to_user_nfts(env: &Env, user: &Address, token_id: &String) {
        let mut nfts: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::UserNfts(user.clone()))
            .unwrap_or(Vec::new(env));
        nfts.push_back(token_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::UserNfts(user.clone()), &nfts);
    }
}

mod test;
