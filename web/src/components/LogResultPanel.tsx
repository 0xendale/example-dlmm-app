export default function LogResultPanel({ result }: { result: any }) {
  if (!result) return null;

  const statusColor =
    result.status === "success" ? "text-emerald-400" : "text-red-400";

  return (
    <div className="mt-6 bg-[#0d0f14]/90 p-5 rounded-xl border border-[#1f2633] shadow-lg font-mono text-sm text-gray-300">
      {/* HEADER */}
      <div className="mb-3">
        <div className="text-lg font-bold text-neon">Transaction logs</div>
        <div className={`mt-1 ${statusColor} font-semibold`}>
          {result.status === "success"
            ? "✓ Swap Executed Successfully"
            : "✗ Swap Failed"}
        </div>
      </div>

      {/* SUMMARY */}
      <div className="space-y-1 mb-4 text-sm">
        <div>
          <span className="text-gray-500">Slot:</span>{" "}
          <span className="text-purple-300">{result.slot}</span>
        </div>

        <div>
          <span className="text-gray-500">Compute Units:</span>{" "}
          <span className="text-blue-300">{result.units}</span>
        </div>

        <div>
          <span className="text-gray-500">Fee:</span>{" "}
          <span className="text-yellow-300">{result.fee}</span>
        </div>

        {result.status === "error" && (
          <div className="text-red-400 mt-2 break-all">
            Error: {JSON.stringify(result.error)}
          </div>
        )}
      </div>

      {/* TERMINAL LOG CONTAINER */}
      <div className="mt-3">
        <div className="font-semibold mb-2 text-gray-400">Program Logs:</div>

        {/* TERMINAL BOX */}
        <div
          className="
            bg-[#05070c]/80 
            p-4 rounded-lg border border-[#1a222d]
            h-72 overflow-y-auto
            font-mono text-xs leading-relaxed 
            shadow-inner
          "
        >
          {result.logs.map((line: string, i: number) => {
            // highlight color
            let cls = "text-gray-300";
            if (line.includes("success")) cls = "text-emerald-400";
            if (line.includes("failed") || line.includes("error"))
              cls = "text-red-400";
            if (line.includes("invoke")) cls = "text-cyan-300";

            return (
              <div key={i} className="whitespace-pre-wrap">
                <span className="text-gray-500">{"> "}</span>
                <span className={cls}>{line}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
