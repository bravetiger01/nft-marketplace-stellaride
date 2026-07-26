"use client";

import { useState, useEffect, useCallback } from "react";
import { getWalletAddress } from "@/hooks/contract";
import {
  mint,
  getNft,
  list,
  delist,
  getListings,
  buy,
  makeOffer,
  getOffer,
  hasOffer,
  cancelOffer,
  getUserNfts,
  createCollection,
  getCollection,
  getCollectionNfts,
  transferNft,
} from "@/hooks/contract";

// ── Types ───────────────────────────────────────────────────────────────────

interface NFTData {
  token_id: string;
  owner: string;
  creator: string;
  metadata_uri: string;
  collection: string;
  royalty_bps: number;
  minted_at: number;
}

interface ListingData {
  token_id: string;
  seller: string;
  price: bigint;
  listed_at: number;
}

// ── Tabs ────────────────────────────────────────────────────────────────────

type Tab = "browse" | "mint" | "my-nfts" | "collections";

// ── Component ───────────────────────────────────────────────────────────────

export default function Contract() {
  const [tab, setTab] = useState<Tab>("browse");
  const [address, setAddress] = useState<string | null>(null);

  useEffect(() => {
    getWalletAddress().then(setAddress);
  }, []);

  return (
    <div className="mx-auto max-w-7xl px-6 py-8">
      {/* Tab Navigation */}
      <div className="mb-8 flex gap-2 border-b border-zinc-800 pb-4">
        {(["browse", "mint", "my-nfts", "collections"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`rounded-lg px-5 py-2.5 text-sm font-medium transition-colors ${
              tab === t
                ? "bg-violet-600 text-white"
                : "text-zinc-400 hover:bg-zinc-800 hover:text-white"
            }`}
          >
            {t === "browse"
              ? "Browse"
              : t === "mint"
              ? "Mint NFT"
              : t === "my-nfts"
              ? "My NFTs"
              : "Collections"}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      {tab === "browse" && <BrowseTab address={address} />}
      {tab === "mint" && <MintTab address={address} />}
      {tab === "my-nfts" && <MyNftsTab address={address} />}
      {tab === "collections" && <CollectionsTab address={address} />}
    </div>
  );
}

// ── Browse Tab ──────────────────────────────────────────────────────────────

function BrowseTab({ address }: { address: string | null }) {
  const [listings, setListings] = useState<ListingData[]>([]);
  const [loading, setLoading] = useState(true);
  const [buying, setBuying] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const items = await getListings(0, 50);
      setListings(items);
    } catch (e) {
      console.error("Failed to load listings:", e);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleBuy = async (tokenId: string) => {
    if (!address) return alert("Connect wallet first");
    setBuying(tokenId);
    try {
      await buy(address, tokenId);
      alert("NFT purchased!");
      load();
    } catch (e: any) {
      alert(`Purchase failed: ${e?.message || e}`);
    }
    setBuying(null);
  };

  return (
    <div>
      <h2 className="mb-6 text-2xl font-bold text-white">Marketplace</h2>
      {loading ? (
        <div className="py-20 text-center text-zinc-400">Loading listings...</div>
      ) : listings.length === 0 ? (
        <div className="py-20 text-center text-zinc-500">
          No listings yet. Mint an NFT and list it for sale!
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {listings.map((l) => (
            <ListingCard
              key={l.token_id}
              listing={l}
              address={address}
              buying={buying === l.token_id}
              onBuy={() => handleBuy(l.token_id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function ListingCard({
  listing,
  address,
  buying,
  onBuy,
}: {
  listing: ListingData;
  address: string | null;
  buying: boolean;
  onBuy: () => void;
}) {
  const [nft, setNft] = useState<NFTData | null>(null);

  useEffect(() => {
    getNft(listing.token_id).then(setNft).catch(console.error);
  }, [listing.token_id]);

  const shortSeller = `${listing.seller.slice(0, 6)}...${listing.seller.slice(-4)}`;
  const isOwner = address === listing.seller;
  const priceXlm = Number(listing.price) / 10_000_000;

  return (
    <div className="overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900 transition-colors hover:border-zinc-700">
      <div className="flex h-48 items-center justify-center bg-gradient-to-br from-violet-900/40 to-fuchsia-900/40">
        <span className="text-5xl">🖼️</span>
      </div>
      <div className="p-5">
        <h3 className="mb-1 text-lg font-semibold text-white">
          {listing.token_id}
        </h3>
        {nft && (
          <p className="mb-3 text-sm text-zinc-400">
            {nft.collection} · {nft.royalty_bps / 100}% royalty
          </p>
        )}
        <p className="mb-1 text-xs text-zinc-500">Seller: {shortSeller}</p>
        <div className="mt-4 flex items-center justify-between">
          <span className="text-xl font-bold text-white">
            {priceXlm.toLocaleString()} XLM
          </span>
          {!isOwner && address && (
            <button
              onClick={onBuy}
              disabled={buying}
              className="rounded-lg bg-violet-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-violet-500 disabled:opacity-50"
            >
              {buying ? "Buying..." : "Buy Now"}
            </button>
          )}
          {isOwner && (
            <span className="text-sm text-zinc-500">You own this</span>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Mint Tab ────────────────────────────────────────────────────────────────

function MintTab({ address }: { address: string | null }) {
  const [tokenId, setTokenId] = useState("");
  const [metadataUri, setMetadataUri] = useState("");
  const [collection, setCollection] = useState("Default");
  const [royaltyBps, setRoyaltyBps] = useState("100");
  const [minting, setMinting] = useState(false);

  const handleMint = async () => {
    if (!address) return alert("Connect wallet first");
    if (!tokenId || !metadataUri) return alert("Fill in all fields");
    setMinting(true);
    try {
      await mint(
        address,
        tokenId,
        metadataUri,
        collection,
        parseInt(royaltyBps) || 0
      );
      alert("NFT minted!");
      setTokenId("");
      setMetadataUri("");
    } catch (e: any) {
      alert(`Mint failed: ${e?.message || e}`);
    }
    setMinting(false);
  };

  return (
    <div className="mx-auto max-w-lg">
      <h2 className="mb-6 text-2xl font-bold text-white">Mint New NFT</h2>
      {!address ? (
        <p className="text-zinc-400">Connect your wallet to mint.</p>
      ) : (
        <div className="space-y-4">
          <Field
            label="Token ID"
            value={tokenId}
            onChange={setTokenId}
            placeholder="e.g. NFT-001"
          />
          <Field
            label="Metadata URI"
            value={metadataUri}
            onChange={setMetadataUri}
            placeholder="e.g. ipfs://Qm..."
          />
          <Field
            label="Collection"
            value={collection}
            onChange={setCollection}
            placeholder="e.g. Stellar Punks"
          />
          <Field
            label="Royalty (basis points, max 1000)"
            value={royaltyBps}
            onChange={setRoyaltyBps}
            placeholder="e.g. 100 = 1%"
          />
          <button
            onClick={handleMint}
            disabled={minting}
            className="w-full rounded-lg bg-gradient-to-r from-violet-500 to-fuchsia-500 py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90 disabled:opacity-50"
          >
            {minting ? "Minting..." : "Mint NFT"}
          </button>
        </div>
      )}
    </div>
  );
}

// ── My NFTs Tab ─────────────────────────────────────────────────────────────

function MyNftsTab({ address }: { address: string | null }) {
  const [nftIds, setNftIds] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [listTokenId, setListTokenId] = useState("");
  const [listPrice, setListPrice] = useState("");
  const [listing, setListing] = useState(false);
  const [transferTo, setTransferTo] = useState("");
  const [transferTokenId, setTransferTokenId] = useState("");
  const [transferring, setTransferring] = useState(false);

  const load = useCallback(async () => {
    if (!address) return;
    setLoading(true);
    try {
      const ids = await getUserNfts(address);
      setNftIds(ids);
    } catch (e) {
      console.error("Failed to load NFTs:", e);
    }
    setLoading(false);
  }, [address]);

  useEffect(() => {
    load();
  }, [load]);

  const handleList = async () => {
    if (!address || !listTokenId || !listPrice) return;
    setListing(true);
    try {
      const priceLamports = BigInt(Math.floor(parseFloat(listPrice) * 10_000_000));
      await list(address, address, listTokenId, priceLamports);
      alert("NFT listed!");
      setListTokenId("");
      setListPrice("");
      load();
    } catch (e: any) {
      alert(`List failed: ${e?.message || e}`);
    }
    setListing(false);
  };

  const handleTransfer = async () => {
    if (!address || !transferTokenId || !transferTo) return;
    setTransferring(true);
    try {
      await transferNft(address, address, transferTo, transferTokenId);
      alert("NFT transferred!");
      setTransferTokenId("");
      setTransferTo("");
      load();
    } catch (e: any) {
      alert(`Transfer failed: ${e?.message || e}`);
    }
    setTransferring(false);
  };

  return (
    <div>
      <h2 className="mb-6 text-2xl font-bold text-white">My NFTs</h2>
      {!address ? (
        <p className="text-zinc-400">Connect your wallet to view your NFTs.</p>
      ) : loading ? (
        <div className="py-10 text-center text-zinc-400">Loading...</div>
      ) : nftIds.length === 0 ? (
        <div className="py-10 text-center text-zinc-500">
          You don&apos;t own any NFTs yet.
        </div>
      ) : (
        <div className="mb-10 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
          {nftIds.map((id) => (
            <NftMiniCard key={id} tokenId={id} />
          ))}
        </div>
      )}

      {/* List for Sale */}
      <div className="mt-8 rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
        <h3 className="mb-4 text-lg font-semibold text-white">List for Sale</h3>
        <div className="flex flex-col gap-3 sm:flex-row">
          <input
            type="text"
            placeholder="Token ID"
            value={listTokenId}
            onChange={(e) => setListTokenId(e.target.value)}
            className="flex-1 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2.5 text-sm text-white placeholder-zinc-500 focus:border-violet-500 focus:outline-none"
          />
          <input
            type="number"
            placeholder="Price (XLM)"
            value={listPrice}
            onChange={(e) => setListPrice(e.target.value)}
            className="w-40 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2.5 text-sm text-white placeholder-zinc-500 focus:border-violet-500 focus:outline-none"
          />
          <button
            onClick={handleList}
            disabled={listing}
            className="rounded-lg bg-violet-600 px-6 py-2.5 text-sm font-semibold text-white hover:bg-violet-500 disabled:opacity-50"
          >
            {listing ? "Listing..." : "List"}
          </button>
        </div>
      </div>

      {/* Transfer */}
      <div className="mt-4 rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
        <h3 className="mb-4 text-lg font-semibold text-white">Transfer NFT</h3>
        <div className="flex flex-col gap-3 sm:flex-row">
          <input
            type="text"
            placeholder="Token ID"
            value={transferTokenId}
            onChange={(e) => setTransferTokenId(e.target.value)}
            className="flex-1 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2.5 text-sm text-white placeholder-zinc-500 focus:border-violet-500 focus:outline-none"
          />
          <input
            type="text"
            placeholder="Recipient address"
            value={transferTo}
            onChange={(e) => setTransferTo(e.target.value)}
            className="flex-1 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2.5 text-sm text-white placeholder-zinc-500 focus:border-violet-500 focus:outline-none"
          />
          <button
            onClick={handleTransfer}
            disabled={transferring}
            className="rounded-lg bg-fuchsia-600 px-6 py-2.5 text-sm font-semibold text-white hover:bg-fuchsia-500 disabled:opacity-50"
          >
            {transferring ? "Sending..." : "Transfer"}
          </button>
        </div>
      </div>
    </div>
  );
}

function NftMiniCard({ tokenId }: { tokenId: string }) {
  const [nft, setNft] = useState<NFTData | null>(null);
  useEffect(() => {
    getNft(tokenId).then(setNft).catch(console.error);
  }, [tokenId]);

  return (
    <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900 p-4">
      <div className="mb-3 flex h-24 items-center justify-center rounded-lg bg-gradient-to-br from-violet-900/40 to-fuchsia-900/40 text-3xl">
        🖼️
      </div>
      <p className="truncate text-sm font-semibold text-white">{tokenId}</p>
      {nft && (
        <p className="truncate text-xs text-zinc-500">{nft.collection}</p>
      )}
    </div>
  );
}

// ── Collections Tab ─────────────────────────────────────────────────────────

function CollectionsTab({ address }: { address: string | null }) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [imageUri, setImageUri] = useState("");
  const [creating, setCreating] = useState(false);
  const [lookupName, setLookupName] = useState("");
  const [foundCollection, setFoundCollection] = useState<any>(null);
  const [collectionNfts, setCollectionNfts] = useState<string[]>([]);

  const handleCreate = async () => {
    if (!address || !name) return alert("Connect wallet and enter a name");
    setCreating(true);
    try {
      await createCollection(address, address, name, description, imageUri);
      alert("Collection created!");
      setName("");
      setDescription("");
      setImageUri("");
    } catch (e: any) {
      alert(`Create failed: ${e?.message || e}`);
    }
    setCreating(false);
  };

  const handleLookup = async () => {
    if (!lookupName) return;
    try {
      const col = await getCollection(lookupName);
      setFoundCollection(col);
      const nfts = await getCollectionNfts(lookupName);
      setCollectionNfts(nfts);
    } catch {
      setFoundCollection(null);
      setCollectionNfts([]);
    }
  };

  return (
    <div>
      <h2 className="mb-6 text-2xl font-bold text-white">Collections</h2>

      {/* Create Collection */}
      <div className="mx-auto max-w-lg rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
        <h3 className="mb-4 text-lg font-semibold text-white">
          Create Collection
        </h3>
        {!address ? (
          <p className="text-zinc-400">Connect your wallet first.</p>
        ) : (
          <div className="space-y-3">
            <Field label="Name" value={name} onChange={setName} placeholder="e.g. Stellar Punks" />
            <Field label="Description" value={description} onChange={setDescription} placeholder="Description..." />
            <Field label="Image URI" value={imageUri} onChange={setImageUri} placeholder="ipfs://..." />
            <button
              onClick={handleCreate}
              disabled={creating}
              className="w-full rounded-lg bg-gradient-to-r from-violet-500 to-fuchsia-500 py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90 disabled:opacity-50"
            >
              {creating ? "Creating..." : "Create Collection"}
            </button>
          </div>
        )}
      </div>

      {/* Lookup Collection */}
      <div className="mx-auto mt-8 max-w-lg rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
        <h3 className="mb-4 text-lg font-semibold text-white">
          Look Up Collection
        </h3>
        <div className="flex gap-3">
          <input
            type="text"
            placeholder="Collection name"
            value={lookupName}
            onChange={(e) => setLookupName(e.target.value)}
            className="flex-1 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2.5 text-sm text-white placeholder-zinc-500 focus:border-violet-500 focus:outline-none"
          />
          <button
            onClick={handleLookup}
            className="rounded-lg bg-zinc-700 px-6 py-2.5 text-sm font-semibold text-white hover:bg-zinc-600"
          >
            Search
          </button>
        </div>
        {foundCollection && (
          <div className="mt-4 rounded-xl bg-zinc-800 p-4">
            <p className="font-semibold text-white">{foundCollection.name}</p>
            <p className="mt-1 text-sm text-zinc-400">
              {foundCollection.description}
            </p>
            <p className="mt-2 text-xs text-zinc-500">
              NFTs: {collectionNfts.length} · Creator:{" "}
              {foundCollection.creator.slice(0, 8)}...
            </p>
            {collectionNfts.length > 0 && (
              <div className="mt-3 flex flex-wrap gap-2">
                {collectionNfts.map((id) => (
                  <span
                    key={id}
                    className="rounded-md bg-violet-900/50 px-2 py-1 text-xs text-violet-300"
                  >
                    {id}
                  </span>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// ── Shared ──────────────────────────────────────────────────────────────────

function Field({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
}) {
  return (
    <div>
      <label className="mb-1 block text-sm font-medium text-zinc-300">
        {label}
      </label>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2.5 text-sm text-white placeholder-zinc-500 focus:border-violet-500 focus:outline-none"
      />
    </div>
  );
}
