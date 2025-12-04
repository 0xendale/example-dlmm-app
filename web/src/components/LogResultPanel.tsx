import React from "react";

export default function LogResultPanel({ result }: { result: any }) {
  if (!result) return null;

  const statusColor =
    result.status === "success" ? "text-emerald-400" : "text-red-400";

  return (
    <div className="mt-6 bg-gray-900/60 p-5 rounded-lg border border-gray-800 font-mono text-sm text-gray-300">
      <div className="text-lg mb-2">
        <span className={statusColor}>
          {result.status === "success"
            ? "✓ Simulation Success"
            : "✗ Simulation Failed"}
        </span>
      </div>

      {/* Summary */}
      <div className="space-y-1 mb-4">
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

        {result.error && (
          <div className="text-red-400 mt-2">
            Error: {JSON.stringify(result.error)}
          </div>
        )}
      </div>

      {/* Logs */}
      <div className="mt-3">
        <div className="font-semibold mb-2 text-gray-400">Program Logs:</div>
        <div className="bg-black/40 p-3 rounded-md border border-gray-700 h-48 overflow-auto">
          {result.response.logs.map((line: string, i: number) => (
            <div key={i} className="text-xs whitespace-pre-wrap">
              {line.includes("failed") || line.includes("error") ? (
                <span className="text-red-400">{line}</span>
              ) : line.includes("success") ? (
                <span className="text-emerald-400">{line}</span>
              ) : (
                line
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
