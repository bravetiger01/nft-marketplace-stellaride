"use client";

import {
  nativeToScVal,
  scValToNative,
  Address,
  xdr,
  Contract as StellarContract,
  TransactionBuilder,
  TimeoutInfinite,
  Keypair,
} from "@stellar/stellar-sdk";
import { Server, assembleTransaction } from "@stellar/stellar-sdk/rpc";
import {
  isConnected,
  isAllowed,
  requestAccess,
  getAddress,
  signTransaction,
} from "@stellar/freighter-api";

// ── Config ──────────────────────────────────────────────────────────────────

const RPC_URL = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = "Test SDF Network ; September 2015";

// ⚠️ UPDATE THIS after deploying the contract
export const CONTRACT_ADDRESS =
  "CCEFWVRY6ODJRWYK56R23FSRW6CYXQJDEZSVLM4QH3AERER56HJXMHEI";

const server = new Server(RPC_URL);

// ── ScVal Converters ────────────────────────────────────────────────────────

function toScValString(v: string) {
  return nativeToScVal(v, { type: "string" });
}

function toScValU32(v: number) {
  return nativeToScVal(v, { type: "u32" });
}

function toScValI128(v: bigint | number) {
  return nativeToScVal(v.toString(), { type: "i128" });
}

function toScValAddress(v: string) {
  return new Address(v).toScVal();
}

function toScValU64(v: bigint | number) {
  return nativeToScVal(v.toString(), { type: "u64" });
}

// ── Wallet ──────────────────────────────────────────────────────────────────

export async function getWalletConnected(): Promise<boolean> {
  const conn = await isConnected();
  return conn.isConnected;
}

export async function connectWallet(): Promise<string> {
  let conn;
  try {
    conn = await isConnected();
  } catch {
    throw new Error(
      "Freyer wallet not found. Install the Freighter browser extension: https://freighter.app"
    );
  }
  if (conn.error) {
    throw new Error(`Freyer error: ${conn.error}`);
  }
  if (!conn.isConnected) {
    throw new Error(
      "Freyer wallet not connected. Open the Freighter extension and connect."
    );
  }
  const allowed = await isAllowed();
  if (allowed.error) {
    throw new Error(`Freyer error: ${allowed.error}`);
  }
  if (!allowed.isAllowed) {
    const access = await requestAccess();
    if (access.error) {
      throw new Error(`Freyer access denied: ${access.error}`);
    }
    return access.address;
  }
  const { address, error } = await getAddress();
  if (error) {
    throw new Error(`Freyer error: ${error}`);
  }
  return address;
}

export async function getWalletAddress(): Promise<string | null> {
  try {
    const conn = await isConnected();
    if (!conn.isConnected) return null;
    const allowed = await isAllowed();
    if (!allowed.isAllowed) return null;
    const { address } = await getAddress();
    return address;
  } catch {
    return null;
  }
}

// ── Core Callers ────────────────────────────────────────────────────────────

async function readContract(
  method: string,
  args: xdr.ScVal[] = [],
  source?: string
) {
  const contract = new StellarContract(CONTRACT_ADDRESS);
  const contractAddr = contract.call(method, ...args);

  let builtTx;
  if (source) {
    const account = await server.getAccount(source);
    const txBuilder = new TransactionBuilder(account, {
      fee: "100",
      networkPassphrase: NETWORK_PASSPHRASE,
    });
    builtTx = txBuilder.addOperation(contractAddr).setTimeout(TimeoutInfinite).build();
  } else {
    // Simulate without a source account (read-only) using a random keypair
    const dummyKeypair = Keypair.random();
    const simulationAccount = {
      accountId: () => dummyKeypair.publicKey(),
      sequenceNumber: () => BigInt(0),
      incrementSequenceNumber: () => {},
    } as any;
    const txBuilder = new TransactionBuilder(simulationAccount, {
      fee: "0",
      networkPassphrase: NETWORK_PASSPHRASE,
    });
    builtTx = txBuilder.addOperation(contractAddr).setTimeout(TimeoutInfinite).build();
  }

  const simResult = await server.simulateTransaction(builtTx);
  if ("result" in simResult && simResult.result) {
    return simResult.result.retval;
  }
  throw new Error(`Simulation failed for ${method}`);
}

async function callContract(
  method: string,
  args: xdr.ScVal[] = [],
  source: string,
  _authRequired: boolean = true
) {
  const account = await server.getAccount(source);
  const contract = new StellarContract(CONTRACT_ADDRESS);
  const contractAddr = contract.call(method, ...args);

  const txBuilder = new TransactionBuilder(account, {
    fee: "1000000",
    networkPassphrase: NETWORK_PASSPHRASE,
  });
  const builtTx = txBuilder.addOperation(contractAddr).setTimeout(TimeoutInfinite).build();

  const simResult = await server.simulateTransaction(builtTx);
  if ("error" in simResult) {
    throw new Error(`Simulation error: ${JSON.stringify(simResult.error)}`);
  }

  const assembledTx = assembleTransaction(
    builtTx,
    simResult
  ).build();

  const signedResult = await signTransaction(
    assembledTx.toXDR(),
    { networkPassphrase: NETWORK_PASSPHRASE }
  );

  const tx = TransactionBuilder.fromXDR(
    signedResult.signedTxXdr,
    NETWORK_PASSPHRASE
  );

  const result = await server.sendTransaction(tx);
  return result;
}

// ── Admin ───────────────────────────────────────────────────────────────────

export async function initialize(
  caller: string,
  admin: string,
  feeBps: number,
  paymentToken: string
) {
  return callContract(
    "initialize",
    [toScValAddress(admin), toScValU32(feeBps), toScValAddress(paymentToken)],
    caller
  );
}

export async function getFee() {
  const result = await readContract("get_fee");
  return scValToNative(result) as number;
}

export async function getAdmin() {
  const result = await readContract("get_admin");
  return scValToNative(result) as string;
}

export async function getPaymentToken() {
  const result = await readContract("get_payment_token");
  return scValToNative(result) as string;
}

export async function setFee(caller: string, newFee: number) {
  return callContract("set_fee", [toScValU32(newFee)], caller);
}

// ── NFT Minting ─────────────────────────────────────────────────────────────

export async function mint(
  caller: string,
  tokenId: string,
  metadataUri: string,
  collection: string,
  royaltyBps: number
) {
  return callContract(
    "mint",
    [
      toScValAddress(caller),
      toScValString(tokenId),
      toScValString(metadataUri),
      toScValString(collection),
      toScValU32(royaltyBps),
    ],
    caller
  );
}

export async function getNft(tokenId: string) {
  const result = await readContract("get_nft", [toScValString(tokenId)]);
  const raw = scValToNative(result) as Record<string, unknown>;
  return {
    token_id: raw.token_id as string,
    owner: raw.owner as string,
    creator: raw.creator as string,
    metadata_uri: raw.metadata_uri as string,
    collection: raw.collection as string,
    royalty_bps: raw.royalty_bps as number,
    minted_at: Number(raw.minted_at as bigint),
  };
}

// ── Transfer ────────────────────────────────────────────────────────────────

export async function transferNft(
  caller: string,
  from: string,
  to: string,
  tokenId: string
) {
  return callContract(
    "transfer_nft",
    [toScValAddress(from), toScValAddress(to), toScValString(tokenId)],
    caller
  );
}

// ── Listing ─────────────────────────────────────────────────────────────────

export async function list(
  caller: string,
  seller: string,
  tokenId: string,
  price: bigint
) {
  return callContract(
    "list",
    [toScValAddress(seller), toScValString(tokenId), toScValI128(price)],
    caller
  );
}

export async function delist(caller: string, seller: string, tokenId: string) {
  return callContract(
    "delist",
    [toScValAddress(seller), toScValString(tokenId)],
    caller
  );
}

export async function getListings(offset: number, limit: number) {
  const result = await readContract("get_listings", [
    toScValU32(offset),
    toScValU32(limit),
  ]);
  const raw = scValToNative(result) as [string, Record<string, unknown>][];
  return raw.map(([tokenId, listing]) => ({
    token_id: tokenId,
    seller: listing.seller as string,
    price: BigInt(listing.price as string | number),
    listed_at: Number(listing.listed_at as bigint),
  }));
}

export async function hasListing(tokenId: string) {
  const result = await readContract("has_listing", [toScValString(tokenId)]);
  return scValToNative(result) as boolean;
}

// ── Purchase ────────────────────────────────────────────────────────────────

export async function buy(caller: string, tokenId: string) {
  return callContract(
    "buy",
    [toScValAddress(caller), toScValString(tokenId)],
    caller
  );
}

// ── Offers ──────────────────────────────────────────────────────────────────

export async function makeOffer(
  caller: string,
  buyer: string,
  tokenId: string,
  amount: bigint,
  expiresAt: bigint
) {
  return callContract(
    "make_offer",
    [
      toScValAddress(buyer),
      toScValString(tokenId),
      toScValI128(amount),
      toScValU64(expiresAt),
    ],
    caller
  );
}

export async function acceptOffer(
  caller: string,
  seller: string,
  tokenId: string,
  buyer: string
) {
  return callContract(
    "accept_offer",
    [
      toScValAddress(seller),
      toScValString(tokenId),
      toScValAddress(buyer),
    ],
    caller
  );
}

export async function cancelOffer(
  caller: string,
  buyer: string,
  tokenId: string
) {
  return callContract(
    "cancel_offer",
    [toScValAddress(buyer), toScValString(tokenId)],
    caller
  );
}

export async function getOffer(tokenId: string, buyer: string) {
  const result = await readContract("get_offer", [
    toScValString(tokenId),
    toScValAddress(buyer),
  ]);
  const raw = scValToNative(result) as Record<string, unknown>;
  return {
    buyer: raw.buyer as string,
    amount: BigInt(raw.amount as string | number),
    expires_at: Number(raw.expires_at as bigint),
  };
}

export async function hasOffer(tokenId: string, buyer: string) {
  const result = await readContract("has_offer", [
    toScValString(tokenId),
    toScValAddress(buyer),
  ]);
  return scValToNative(result) as boolean;
}

// ── Collections ─────────────────────────────────────────────────────────────

export async function createCollection(
  caller: string,
  creator: string,
  name: string,
  description: string,
  imageUri: string
) {
  return callContract(
    "create_collection",
    [
      toScValAddress(creator),
      toScValString(name),
      toScValString(description),
      toScValString(imageUri),
    ],
    caller
  );
}

export async function getCollection(name: string) {
  const result = await readContract("get_collection", [toScValString(name)]);
  const raw = scValToNative(result) as Record<string, unknown>;
  return {
    name: raw.name as string,
    creator: raw.creator as string,
    description: raw.description as string,
    image_uri: raw.image_uri as string,
  };
}

// ── Queries ─────────────────────────────────────────────────────────────────

export async function getUserNfts(user: string) {
  const result = await readContract("get_user_nfts", [toScValAddress(user)]);
  return scValToNative(result) as string[];
}

export async function getCollectionNfts(name: string) {
  const result = await readContract("get_collection_nfts", [
    toScValString(name),
  ]);
  return scValToNative(result) as string[];
}
