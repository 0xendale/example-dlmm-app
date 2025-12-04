import { useEffect, useState } from "react";
import SwapInstruction from "./Swap";
import CreatePositionInstruction from "./CreatePosition";
import ModifyPositionInstruction from "./ModifierPosition";
import { TokenInfo } from "../../utils/type";

const tabs = [
  { key: "create", label: "Create Position" },
  { key: "modify", label: "Modify Position" },
];

type Props = {
  pairAddress: string;
  tokens: { a: TokenInfo; b: TokenInfo };
  setPair: (v: string) => void;
  fetchPair: (address: string) => Promise<void>;
  pairStatus: string;
  instructionTab: string;
  loading?: boolean;
  swapParams?: any;
};

export default function InstructionTabs({
  pairAddress,
  tokens,
  setPair,
  loading,
  instructionTab,
  swapParams,
}: Props) {
  const [active, setActive] = useState("swap_instruction");
  const [instruction, setInstruction] = useState("create");
  // useEffect(() => {
  //   fetchInstruction(pairAddress);
  // }, [pairAddress]);

  useEffect(() => {
    if (instructionTab === "swap") setActive("swap_instruction");
    if (instructionTab === "position") setActive("position_manager");
  }, [instructionTab]);

  const fetchInstruction = async (
    address: String,
    instructionType: string,
    params: any
  ) => {
    // Fetch instruction details from backend API
    try {
      const res = await fetch("/api/instruction", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          pair_address: address,
          instruction_type: instructionType,
          params: params,
        }),
      });
      const data = await res.json();
      console.log("Fetched instruction data:", data);
      // Handle the fetched instruction data as needed
    } catch (err) {
      console.error("Error fetching instruction data:", err);
    }
  };

  return (
    <div className="box-middle mt-10">
      <h2 className="inst-title fancy">
        Instruction Details
        <span className="inst-sweep" />
      </h2>

      {/* === Content === */}
      <div className="inst-content">
        {active === "swap_instruction" && (
          <SwapInstruction
            pairAddress={pairAddress}
            tokens={tokens}
            fetchInstruction={fetchInstruction}
            swapParams={swapParams}
          />
        )}

        {active === "position_manager" && (
          <div>
            {/* === Tabs header === */}
            <div className="inst-bar">
              {tabs.map((t) => (
                <button
                  key={t.key}
                  onClick={() => setInstruction(t.key)}
                  className={`inst-btn ${
                    instruction === t.key ? "inst-active" : ""
                  }`}
                >
                  {t.label}
                </button>
              ))}
            </div>
            {instruction === "create" && (
              <CreatePositionInstruction
                pairAddress={pairAddress}
                setPair={setPair}
                getInstruction={fetchInstruction}
              />
            )}
            {instruction === "modify" && <ModifyPositionInstruction />}
          </div>
        )}
        {active === "modify" && <ModifyPositionInstruction />}
      </div>
    </div>
  );
}
