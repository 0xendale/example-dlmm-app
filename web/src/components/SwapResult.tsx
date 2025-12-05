import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useState } from "react";
import LogResultPanel from "./LogResultPanel";

type Props = {
  loading?: boolean;
  txSwap: any;
};

export default function SwapResult({ loading, txSwap }: Props) {
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
                Swaping...
              </span>
            </motion.div>
          )}

          {/* Error state */}
          {!loading && txSwap?.error && (
            <motion.div
              key="error"
              initial={{ opacity: 0, y: -8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              className="mt-6 p-4 bg-gradient-to-r from-red-900/60 to-red-700/30 border border-red-600/60 text-red-300 rounded-lg text-sm font-mono"
            >
              <div className="flex items-center gap-2">
                <span className="text-red-400">⚠️</span>
                <span>{txSwap.error}</span>
              </div>
            </motion.div>
          )}

          {/* Swap execution result */}
          {!loading && !txSwap?.error && txSwap && (
            <motion.div
              key="swap-result"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.25 }}
              className="mt-8"
            >
              <LogResultPanel result={txSwap} />
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </>
  );
}
