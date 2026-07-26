#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

// ── Helpers ────────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    NftMarketplaceClient<'static>,
    StellarAssetClient<'static>,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_sac = env.register_stellar_asset_contract_v2(token_admin);
    let token_addr = token_sac.address();
    let token = StellarAssetClient::new(&env, &token_addr);

    let contract_id = env.register(NftMarketplace, ());
    let client = NftMarketplaceClient::new(&env, &contract_id);
    client.initialize(&admin, &250, &token_addr); // 2.5% fee

    (env, client, token, admin)
}

// ── Initialize Tests ───────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let (_env, client, _, admin) = setup();
    assert_eq!(client.get_fee(), 250);
    assert_eq!(client.get_admin(), admin);
    assert_eq!(
        client.get_payment_token(),
        client.get_payment_token()
    );
}

#[test]
fn test_set_fee() {
    let (_env, client, _, _) = setup();
    client.set_fee(&500);
    assert_eq!(client.get_fee(), 500);
}

#[test]
#[should_panic(expected = "fee max 10%")]
fn test_set_fee_too_high() {
    let (_env, client, _, _) = setup();
    client.set_fee(&1001);
}

// ── Mint Tests ─────────────────────────────────────────────────────────────

#[test]
fn test_mint_nft() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);

    client.mint(
        &user,
        &String::from_str(&env, "NFT-001"),
        &String::from_str(&env, "ipfs://QmTest123"),
        &String::from_str(&env, "Stellar Punks"),
        &100,
    );

    let nft = client.get_nft(&String::from_str(&env, "NFT-001"));
    assert_eq!(nft.token_id, String::from_str(&env, "NFT-001"));
    assert_eq!(nft.owner, user);
    assert_eq!(nft.creator, user);
    assert_eq!(nft.collection, String::from_str(&env, "Stellar Punks"));
    assert_eq!(nft.royalty_bps, 100);
    assert_eq!(nft.metadata_uri, String::from_str(&env, "ipfs://QmTest123"));
}

#[test]
fn test_mint_tracks_in_user_nfts() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);

    client.mint(
        &user,
        &String::from_str(&env, "NFT-001"),
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );

    let nfts = client.get_user_nfts(&user);
    assert_eq!(nfts.len(), 1);
    assert_eq!(
        nfts.get(0),
        Some(String::from_str(&env, "NFT-001"))
    );
}

#[test]
fn test_mint_tracks_in_collection() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);
    let col = String::from_str(&env, "Stellar Punks");

    client.mint(
        &user,
        &String::from_str(&env, "NFT-001"),
        &String::from_str(&env, "ipfs://test"),
        &col,
        &0,
    );
    client.mint(
        &user,
        &String::from_str(&env, "NFT-002"),
        &String::from_str(&env, "ipfs://test2"),
        &col,
        &0,
    );

    let col_nfts = client.get_collection_nfts(&col);
    assert_eq!(col_nfts.len(), 2);
}

#[test]
#[should_panic(expected = "already minted")]
fn test_mint_duplicate() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");
    let uri = String::from_str(&env, "ipfs://test");
    let col = String::from_str(&env, "Col");

    client.mint(&user, &token_id, &uri, &col, &0);
    client.mint(&user, &token_id, &uri, &col, &0);
}

#[test]
#[should_panic(expected = "royalty max 10%")]
fn test_mint_royalty_too_high() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);

    client.mint(
        &user,
        &String::from_str(&env, "NFT-001"),
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &1001,
    );
}

// ── Transfer Tests ─────────────────────────────────────────────────────────

#[test]
fn test_transfer_nft() {
    let (env, client, _, _) = setup();
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &from,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.transfer_nft(&from, &to, &token_id);

    let nft = client.get_nft(&token_id);
    assert_eq!(nft.owner, to);

    let from_nfts = client.get_user_nfts(&from);
    assert_eq!(from_nfts.len(), 0);
    let to_nfts = client.get_user_nfts(&to);
    assert_eq!(to_nfts.len(), 1);
}

#[test]
#[should_panic(expected = "not owner")]
fn test_transfer_not_owner() {
    let (env, client, _, _) = setup();
    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &owner,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.transfer_nft(&other, &Address::generate(&env), &token_id);
}

// ── Listing Tests ──────────────────────────────────────────────────────────

#[test]
fn test_list_nft() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &100,
    );
    client.list(&seller, &token_id, &10_000);

    assert!(client.has_listing(&token_id));
    let listings = client.get_listings(&0, &10);
    assert_eq!(listings.len(), 1);
}

#[test]
fn test_delist_nft() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.list(&seller, &token_id, &10_000);
    client.delist(&seller, &token_id);

    assert!(!client.has_listing(&token_id));
    assert_eq!(client.get_listings(&0, &10).len(), 0);
}

#[test]
#[should_panic(expected = "already listed")]
fn test_list_already_listed() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.list(&seller, &token_id, &10_000);
    client.list(&seller, &token_id, &20_000);
}

#[test]
#[should_panic(expected = "not owner")]
fn test_list_not_owner() {
    let (env, client, _, _) = setup();
    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &owner,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.list(&other, &token_id, &10_000);
}

#[test]
#[should_panic(expected = "price must be positive")]
fn test_list_zero_price() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.list(&seller, &token_id, &0);
}

#[test]
#[should_panic(expected = "not seller")]
fn test_delist_wrong_seller() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);
    let other = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.list(&seller, &token_id, &10_000);
    client.delist(&other, &token_id);
}

// ── Buy Tests ──────────────────────────────────────────────────────────────

#[test]
fn test_buy_nft_basic() {
    let (env, client, token, _) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    token.mint(&buyer, &100_000);

    client.list(&seller, &token_id, &10_000);
    client.buy(&buyer, &token_id);

    let nft = client.get_nft(&token_id);
    assert_eq!(nft.owner, buyer);
    assert!(!client.has_listing(&token_id));

    let buyer_nfts = client.get_user_nfts(&buyer);
    assert_eq!(buyer_nfts.len(), 1);
}

#[test]
fn test_buy_fee_distribution_no_royalty() {
    let (env, client, token, admin) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    // Mint with 0 royalty (creator == seller, so no royalty paid)
    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    token.mint(&buyer, &1_000_000);

    client.list(&seller, &token_id, &100_000);
    client.buy(&buyer, &token_id);

    // 2.5% fee = 2,500 to admin; 0 royalty; seller gets 97,500
    assert_eq!(token.balance(&admin), 2_500);
    assert_eq!(token.balance(&seller), 97_500);
}

#[test]
fn test_buy_with_royalty_different_creator() {
    let (env, client, token, admin) = setup();
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    // Creator mints with 10% royalty
    client.mint(
        &creator,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &1000,
    );
    // Transfer to seller
    client.transfer_nft(&creator, &seller, &token_id);
    // Fund buyer
    token.mint(&buyer, &1_000_000);

    client.list(&seller, &token_id, &100_000);
    client.buy(&buyer, &token_id);

    // 2.5% fee = 2,500 to admin
    // 10% royalty = 10,000 to creator
    // seller gets 100,000 - 2,500 - 10,000 = 87,500
    assert_eq!(token.balance(&admin), 2_500);
    assert_eq!(token.balance(&creator), 10_000);
    assert_eq!(token.balance(&seller), 87_500);
}

#[test]
fn test_buy_creator_is_seller_no_royalty() {
    let (env, client, token, admin) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    // Creator/seller mints with 10% royalty — but creator==seller, so 0 royalty paid
    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &1000,
    );
    token.mint(&buyer, &1_000_000);

    client.list(&seller, &token_id, &100_000);
    client.buy(&buyer, &token_id);

    // 2.5% fee = 2,500; royalty skipped (creator == seller); seller gets 97,500
    assert_eq!(token.balance(&admin), 2_500);
    assert_eq!(token.balance(&seller), 97_500);
}

#[test]
#[should_panic(expected = "not listed")]
fn test_buy_not_listed() {
    let (env, client, _, _) = setup();
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &Address::generate(&env),
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.buy(&buyer, &token_id);
}

#[test]
#[should_panic(expected = "cannot buy own NFT")]
fn test_buy_own_nft() {
    let (env, client, token, _) = setup();
    let seller = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    token.mint(&seller, &1_000_000);
    client.list(&seller, &token_id, &10_000);
    client.buy(&seller, &token_id);
}

#[test]
fn test_buy_removes_listing_on_transfer() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.list(&seller, &token_id, &10_000);

    // Transferring should remove listing
    client.transfer_nft(&seller, &Address::generate(&env), &token_id);
    assert!(!client.has_listing(&token_id));
}

// ── Offer Tests ────────────────────────────────────────────────────────────

#[test]
fn test_make_offer() {
    let (env, client, _, _) = setup();
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &Address::generate(&env),
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    let expires = env.ledger().timestamp() + 1000;
    client.make_offer(&buyer, &token_id, &50_000, &expires);

    assert!(client.has_offer(&token_id, &buyer));
    let offer = client.get_offer(&token_id, &buyer);
    assert_eq!(offer.amount, 50_000);
    assert_eq!(offer.buyer, buyer);
}

#[test]
fn test_accept_offer() {
    let (env, client, token, _) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &seller,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    token.mint(&buyer, &1_000_000);

    let expires = env.ledger().timestamp() + 1000;
    client.make_offer(&buyer, &token_id, &50_000, &expires);
    client.accept_offer(&seller, &token_id, &buyer);

    let nft = client.get_nft(&token_id);
    assert_eq!(nft.owner, buyer);
    assert!(!client.has_offer(&token_id, &buyer));
}

#[test]
fn test_accept_offer_fee_distribution() {
    let (env, client, token, admin) = setup();
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    // 10% royalty, creator != seller
    client.mint(
        &creator,
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &1000,
    );
    client.transfer_nft(&creator, &seller, &token_id);
    token.mint(&buyer, &1_000_000);

    let expires = env.ledger().timestamp() + 1000;
    client.make_offer(&buyer, &token_id, &200_000, &expires);
    client.accept_offer(&seller, &token_id, &buyer);

    // 2.5% fee = 5,000; 10% royalty = 20,000; seller gets 175,000
    assert_eq!(token.balance(&admin), 5_000);
    assert_eq!(token.balance(&creator), 20_000);
    assert_eq!(token.balance(&seller), 175_000);
}

#[test]
fn test_cancel_offer() {
    let (env, client, _, _) = setup();
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &Address::generate(&env),
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    let expires = env.ledger().timestamp() + 1000;
    client.make_offer(&buyer, &token_id, &50_000, &expires);
    client.cancel_offer(&buyer, &token_id);

    assert!(!client.has_offer(&token_id, &buyer));
}

#[test]
#[should_panic(expected = "offer not found")]
fn test_cancel_offer_not_found() {
    let (env, client, _, _) = setup();
    let buyer = Address::generate(&env);
    client.cancel_offer(&buyer, &String::from_str(&env, "NFT-X"));
}

#[test]
#[should_panic(expected = "expires in the past")]
fn test_make_offer_expired() {
    let (env, client, _, _) = setup();
    let buyer = Address::generate(&env);
    let token_id = String::from_str(&env, "NFT-001");

    client.mint(
        &Address::generate(&env),
        &token_id,
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.make_offer(&buyer, &token_id, &50_000, &0);
}

#[test]
#[should_panic(expected = "NFT not found")]
fn test_make_offer_nft_not_found() {
    let (env, client, _, _) = setup();
    let buyer = Address::generate(&env);
    let expires = env.ledger().timestamp() + 1000;

    client.make_offer(
        &buyer,
        &String::from_str(&env, "NFT-X"),
        &50_000,
        &expires,
    );
}

// ── Collection Tests ───────────────────────────────────────────────────────

#[test]
fn test_create_collection() {
    let (env, client, _, _) = setup();
    let creator = Address::generate(&env);

    client.create_collection(
        &creator,
        &String::from_str(&env, "Stellar Punks"),
        &String::from_str(&env, "The best NFTs on Stellar"),
        &String::from_str(&env, "ipfs://col-image"),
    );

    let col = client.get_collection(&String::from_str(&env, "Stellar Punks"));
    assert_eq!(col.creator, creator);
    assert_eq!(
        col.description,
        String::from_str(&env, "The best NFTs on Stellar")
    );
    assert_eq!(
        col.image_uri,
        String::from_str(&env, "ipfs://col-image")
    );

    let col_nfts = client.get_collection_nfts(&String::from_str(&env, "Stellar Punks"));
    assert_eq!(col_nfts.len(), 0);
}

#[test]
#[should_panic(expected = "collection exists")]
fn test_create_collection_duplicate() {
    let (env, client, _, _) = setup();
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Stellar Punks");

    client.create_collection(
        &creator,
        &name,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "img"),
    );
    client.create_collection(
        &creator,
        &name,
        &String::from_str(&env, "desc2"),
        &String::from_str(&env, "img2"),
    );
}

#[test]
#[should_panic(expected = "collection not found")]
fn test_get_collection_not_found() {
    let (env, client, _, _) = setup();
    client.get_collection(&String::from_str(&env, "NonExistent"));
}

// ── Pagination Tests ───────────────────────────────────────────────────────

#[test]
fn test_get_listings_pagination() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);

    let ids = ["NFT-0", "NFT-1", "NFT-2", "NFT-3", "NFT-4"];
    for id in &ids {
        client.mint(
            &seller,
            &String::from_str(&env, id),
            &String::from_str(&env, "ipfs://test"),
            &String::from_str(&env, "Col"),
            &0,
        );
        client.list(&seller, &String::from_str(&env, id), &1_000);
    }

    assert_eq!(client.get_listings(&0, &10).len(), 5);
    assert_eq!(client.get_listings(&0, &2).len(), 2);
    assert_eq!(client.get_listings(&2, &2).len(), 2);
    assert_eq!(client.get_listings(&4, &2).len(), 1);
    assert_eq!(client.get_listings(&5, &2).len(), 0);
}

#[test]
fn test_get_listings_only_active() {
    let (env, client, _, _) = setup();
    let seller = Address::generate(&env);

    client.mint(
        &seller,
        &String::from_str(&env, "NFT-0"),
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );
    client.mint(
        &seller,
        &String::from_str(&env, "NFT-1"),
        &String::from_str(&env, "ipfs://test"),
        &String::from_str(&env, "Col"),
        &0,
    );

    client.list(&seller, &String::from_str(&env, "NFT-0"), &1_000);

    let listings = client.get_listings(&0, &10);
    assert_eq!(listings.len(), 1);
}

// ── Edge Case Tests ────────────────────────────────────────────────────────

#[test]
fn test_empty_user_nfts() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);
    let nfts = client.get_user_nfts(&user);
    assert_eq!(nfts.len(), 0);
}

#[test]
fn test_empty_collection_nfts() {
    let (env, client, _, _) = setup();
    let nfts = client.get_collection_nfts(&String::from_str(&env, "Any"));
    assert_eq!(nfts.len(), 0);
}

#[test]
fn test_empty_listings() {
    let (env, client, _, _) = setup();
    let listings = client.get_listings(&0, &10);
    assert_eq!(listings.len(), 0);
}

// ── Full Lifecycle Test ────────────────────────────────────────────────────

#[test]
fn test_full_lifecycle() {
    let (env, client, token, admin) = setup();
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    // 1. Create collection
    client.create_collection(
        &creator,
        &String::from_str(&env, "My Collection"),
        &String::from_str(&env, "A test collection"),
        &String::from_str(&env, "ipfs://banner"),
    );

    // 2. Mint NFT
    client.mint(
        &creator,
        &String::from_str(&env, "NFT-001"),
        &String::from_str(&env, "ipfs://meta"),
        &String::from_str(&env, "My Collection"),
        &500, // 5% royalty
    );

    // 3. Verify collection has NFT
    let col_nfts = client.get_collection_nfts(&String::from_str(&env, "My Collection"));
    assert_eq!(col_nfts.len(), 1);

    // 4. Fund buyer
    token.mint(&buyer, &1_000_000);

    // 5. Creator lists
    client.list(&creator, &String::from_str(&env, "NFT-001"), &50_000);

    // 6. Buyer makes offer
    let expires = env.ledger().timestamp() + 1000;
    client.make_offer(
        &buyer,
        &String::from_str(&env, "NFT-001"),
        &40_000,
        &expires,
    );

    // 7. Creator accepts offer
    client.accept_offer(
        &creator,
        &String::from_str(&env, "NFT-001"),
        &buyer,
    );

    // 8. Verify ownership
    let nft = client.get_nft(&String::from_str(&env, "NFT-001"));
    assert_eq!(nft.owner, buyer);
    assert_eq!(nft.creator, creator);

    // 9. Verify payments: 2.5% fee = 1,000; 5% royalty = 2,000; seller gets 37,000
    assert_eq!(token.balance(&admin), 1_000);
    assert_eq!(token.balance(&creator), 39_000); // 37,000 + 2,000 royalty
    assert_eq!(token.balance(&buyer), 960_000); // 1,000,000 - 40,000
}
