import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useState } from "react";

type Props = {
  quote: any;
  getSwap: (params: any) => void;
  loading?: boolean;
  defaultSigner?: string;
};

export default function QuoteResult({
  quote,
  getSwap,
  loading,
  defaultSigner = "RFdow49qKnuRLKu24XjJFiyhsXjaUWtzNxeA38t58At",
}: Props) {
  const [swapParams, setSwapParams] = useState<any>(null);
  const [signer, setSigner] = useState<string>(defaultSigner);
  const [signerTouched, setSignerTouched] = useState(false);

  useEffect(() => {
    if (!quote || quote.error) {
      setSwapParams(null);
      return;
    }

    const nextParams = {
      source_mint: quote?.input?.address,
      in_amount: quote?.in_amount,
      min_out_amount: quote?.out_amount,
    };

    setSwapParams(nextParams);
  }, [quote]);

  const handleSwapClick = () => {
    setSignerTouched(true);
    if (!swapParams || !signer.trim()) return;

    getSwap({
      ...swapParams,
      signer,
    });
  };

  const canSwap = !!swapParams && !!signer.trim() && !loading;

  return (
    <>
      <div className="mt-8 min-h-[6rem]">
        {/* Loading spinner */}
        <AnimatePresence mode="wait">
          {loading && (
            <motion.div
              key="loading"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="flex items-center justify-center py-6"
            >
              <div className="w-6 h-6 border-2 border-t-transparent border-neon rounded-full animate-spin" />
              <span className="ml-3 text-neon text-sm font-mono">
                Fetching quote...
              </span>
            </motion.div>
          )}

          {/* Error state */}
          {!loading && quote?.error && (
            <motion.div
              key="error"
              initial={{ opacity: 0, y: -8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              className="mt-6 p-4 bg-gradient-to-r from-red-900/60 to-red-700/30 border border-red-600/60 text-red-300 rounded-lg text-sm font-mono"
            >
              <div className="flex items-center gap-2">
                <span className="text-red-400">⚠️</span>
                <span>{quote.error}</span>
              </div>
            </motion.div>
          )}

          {/* Quote result */}
          {!loading && quote && !quote.error && (
            <motion.div
              key="quote"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.25 }}
              className="mt-6 bg-gray-900/60 p-4 rounded-lg border border-gray-800 font-mono text-sm text-gray-300 space-y-1"
            >
              <div>
                <span className="text-gray-500">Input:</span>{" "}
                <span className="text-cyan-300">
                  {quote?.input?.symbol} — {quote.in_amount}
                </span>
              </div>
              <div>
                <span className="text-gray-500">Output:</span>{" "}
                <span className="text-green-300">
                  {quote?.output?.symbol} — {quote.out_amount}
                </span>
              </div>
              <div>
                <span className="text-gray-500">Fee:</span>{" "}
                <span className="text-yellow-300">{quote.fee_amount}</span>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Signer input + Swap button */}
      <div className="inst-bar flex flex-col sm:flex-row gap-3 items-stretch sm:items-center mt-4">
        <div className="flex-1">
          <label className="block text-xs font-mono text-gray-400 mb-1">
            Signer (wallet address / pubkey)
          </label>
          <input
            value={signer}
            onChange={(e) => setSigner(e.target.value)}
            onBlur={() => setSignerTouched(true)}
            placeholder="Enter signer address"
            className="w-full px-3 py-2 rounded-md bg-gray-900/70 border border-gray-700 text-sm font-mono text-gray-100 placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-neon focus:border-neon"
          />
          {signerTouched && !signer.trim() && (
            <p className="mt-1 text-xs text-red-400 font-mono">
              Signer is required to execute swap.
            </p>
          )}
        </div>

        <button
          onClick={handleSwapClick}
          disabled={!canSwap}
          className={`inst-btn px-4 py-2 rounded-md text-sm font-semibold mt-2 sm:mt-6
            ${
              canSwap
                ? "bg-neon text-black hover:brightness-110"
                : "bg-gray-800 text-gray-500 cursor-not-allowed"
            }`}
        >
          {loading ? "Processing..." : "Get Swap"}
        </button>
      </div>
    </>
  );
}
