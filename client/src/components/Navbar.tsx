"use client";

import { useState, useEffect } from "react";
import { connectWallet, getWalletAddress } from "@/hooks/contract";

export default function Navbar() {
  const [address, setAddress] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getWalletAddress().then(setAddress);
  }, []);

  const handleConnect = async () => {
    setConnecting(true);
    setError(null);
    try {
      const addr = await connectWallet();
      setAddress(addr);
    } catch (e: any) {
      console.error("Wallet connection failed:", e);
      setError(e?.message || "Failed to connect wallet");
    }
    setConnecting(false);
  };

  const shortAddr = address
    ? `${address.slice(0, 6)}...${address.slice(-4)}`
    : null;

  return (
    <nav className="sticky top-0 z-50 border-b border-zinc-800 bg-black/80 backdrop-blur-md">
      <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-fuchsia-500 text-lg font-bold text-white">
            N
          </div>
          <span className="text-xl font-bold text-white">NFT Marketplace</span>
        </div>

        <div className="flex items-center gap-4">
          {address ? (
            <div className="flex items-center gap-3">
              <span className="rounded-full bg-zinc-800 px-4 py-2 font-mono text-sm text-zinc-300">
                {shortAddr}
              </span>
              <div className="h-2 w-2 rounded-full bg-green-400" />
            </div>
          ) : (
            <div className="flex flex-col items-end gap-1">
              <button
                onClick={handleConnect}
                disabled={connecting}
                className="rounded-full bg-gradient-to-r from-violet-500 to-fuchsia-500 px-6 py-2 text-sm font-semibold text-white transition-all hover:opacity-90 disabled:opacity-50"
              >
                {connecting ? "Connecting..." : "Connect Wallet"}
              </button>
              {error && (
                <span className="max-w-xs text-right text-xs text-red-400">
                  {error}
                </span>
              )}
            </div>
          )}
        </div>
      </div>
    </nav>
  );
}
