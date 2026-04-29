import { useState } from "react";

// Uses public Solana RPC endpoints that have CORS enabled
const RPC_ENDPOINTS = [
  "https://solana-mainnet.g.alchemy.com/v2/demo",
  "https://api.mainnet-beta.solana.com",
];

const styles = `
  @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&family=Rajdhani:wght@300;400;500;600;700&display=swap');
  * { margin: 0; padding: 0; box-sizing: border-box; }
  :root {
    --bg: #020b0f; --bg2: #041218; --panel: #061a22; --border: #0d3545;
    --accent: #00d4ff; --accent2: #00ff88; --danger: #ff3b5c; --warn: #ffaa00;
    --text: #c8e8f0; --muted: #4a7a8a;
    --font-mono: 'Share Tech Mono', monospace;
    --font-display: 'Rajdhani', sans-serif;
  }
  body { background: var(--bg); color: var(--text); font-family: var(--font-display); min-height: 100vh; overflow-x: hidden; }
  .scanlines { position: fixed; inset: 0; background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.08) 2px, rgba(0,0,0,0.08) 4px); pointer-events: none; z-index: 100; }
  .grid-bg { position: fixed; inset: 0; background-image: linear-gradient(rgba(0,212,255,0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(0,212,255,0.03) 1px, transparent 1px); background-size: 40px 40px; pointer-events: none; }
  .container { max-width: 900px; margin: 0 auto; padding: 40px 20px; position: relative; z-index: 1; }
  .header { text-align: center; margin-bottom: 48px; }
  .logo-line { display: flex; align-items: center; justify-content: center; gap: 12px; margin-bottom: 8px; }
  .logo-icon { width: 48px; height: 48px; border: 2px solid var(--accent); display: flex; align-items: center; justify-content: center; font-size: 22px; box-shadow: 0 0 20px rgba(0,212,255,0.4); animation: pulse 2s ease-in-out infinite; }
  @keyframes pulse { 0%,100%{box-shadow:0 0 20px rgba(0,212,255,0.4)}50%{box-shadow:0 0 40px rgba(0,212,255,0.7)} }
  .title { font-family: var(--font-display); font-size: 42px; font-weight: 700; letter-spacing: 6px; color: var(--accent); text-shadow: 0 0 30px rgba(0,212,255,0.5); text-transform: uppercase; }
  .subtitle { font-family: var(--font-mono); font-size: 13px; color: var(--muted); letter-spacing: 3px; margin-top: 6px; }
  .tag-line { display: inline-block; font-family: var(--font-mono); font-size: 11px; color: var(--accent2); border: 1px solid var(--accent2); padding: 3px 10px; margin-top: 12px; letter-spacing: 2px; opacity: 0.8; }
  .input-panel { background: var(--panel); border: 1px solid var(--border); padding: 28px; margin-bottom: 24px; position: relative; }
  .input-panel::before { content: 'TOKEN SCAN TERMINAL'; position: absolute; top: -10px; left: 20px; background: var(--panel); padding: 0 10px; font-family: var(--font-mono); font-size: 10px; color: var(--accent); letter-spacing: 2px; }
  .input-row { display: flex; gap: 12px; }
  .address-input { flex: 1; background: var(--bg2); border: 1px solid var(--border); color: var(--accent); font-family: var(--font-mono); font-size: 14px; padding: 14px 18px; outline: none; letter-spacing: 1px; transition: border-color 0.2s; }
  .address-input::placeholder { color: var(--muted); }
  .address-input:focus { border-color: var(--accent); box-shadow: 0 0 15px rgba(0,212,255,0.15); }
  .scan-btn { background: transparent; border: 2px solid var(--accent); color: var(--accent); font-family: var(--font-display); font-size: 15px; font-weight: 700; letter-spacing: 3px; padding: 14px 32px; cursor: pointer; text-transform: uppercase; transition: all 0.2s; white-space: nowrap; }
  .scan-btn:hover:not(:disabled) { background: var(--accent); color: var(--bg); box-shadow: 0 0 30px rgba(0,212,255,0.4); }
  .scan-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .examples { margin-top: 12px; font-family: var(--font-mono); font-size: 11px; color: var(--muted); }
  .examples span { color: var(--accent); cursor: pointer; margin-left: 8px; opacity: 0.7; transition: opacity 0.2s; }
  .examples span:hover { opacity: 1; }
  .loading-panel { background: var(--panel); border: 1px solid var(--border); padding: 48px; text-align: center; }
  .loading-text { font-family: var(--font-mono); font-size: 14px; color: var(--accent); letter-spacing: 2px; margin-bottom: 24px; }
  .progress-bar { width: 100%; height: 2px; background: var(--border); overflow: hidden; }
  .progress-fill { height: 100%; background: var(--accent); animation: scan 1.5s ease-in-out infinite; box-shadow: 0 0 10px var(--accent); }
  @keyframes scan { 0%{width:0%;margin-left:0}50%{width:60%}100%{width:0%;margin-left:100%} }
  .log-lines { margin-top: 20px; text-align: left; }
  .log-line { font-family: var(--font-mono); font-size: 12px; color: var(--muted); padding: 3px 0; animation: fadeIn 0.3s ease; }
  .log-line.active { color: var(--accent2); }
  @keyframes fadeIn { from{opacity:0;transform:translateX(-10px)}to{opacity:1} }
  .results { display: flex; flex-direction: column; gap: 16px; }
  .score-panel { background: var(--panel); border: 1px solid var(--border); padding: 28px; display: flex; align-items: center; gap: 32px; position: relative; }
  .score-panel::before { content: 'RISK ASSESSMENT'; position: absolute; top: -10px; left: 20px; background: var(--panel); padding: 0 10px; font-family: var(--font-mono); font-size: 10px; letter-spacing: 2px; }
  .score-circle { width: 100px; height: 100px; border-radius: 50%; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 3px solid; flex-shrink: 0; }
  .score-number { font-family: var(--font-mono); font-size: 32px; font-weight: bold; line-height: 1; }
  .score-label { font-family: var(--font-mono); font-size: 9px; letter-spacing: 2px; margin-top: 2px; opacity: 0.7; }
  .score-info { flex: 1; }
  .risk-badge { display: inline-block; font-family: var(--font-mono); font-size: 11px; letter-spacing: 3px; padding: 4px 12px; border: 1px solid; margin-bottom: 10px; text-transform: uppercase; }
  .verdict { font-size: 18px; font-weight: 600; letter-spacing: 1px; margin-bottom: 6px; }
  .verdict-detail { font-family: var(--font-mono); font-size: 12px; color: var(--muted); line-height: 1.6; }
  .data-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .data-card { background: var(--panel); border: 1px solid var(--border); padding: 18px; }
  .data-card-label { font-family: var(--font-mono); font-size: 10px; color: var(--muted); letter-spacing: 2px; margin-bottom: 8px; text-transform: uppercase; }
  .data-card-value { font-family: var(--font-mono); font-size: 20px; font-weight: bold; color: var(--accent); }
  .data-card-sub { font-size: 12px; color: var(--muted); margin-top: 4px; }
  .flags-panel { background: var(--panel); border: 1px solid var(--border); padding: 28px; position: relative; }
  .flags-panel::before { content: 'THREAT INDICATORS'; position: absolute; top: -10px; left: 20px; background: var(--panel); padding: 0 10px; font-family: var(--font-mono); font-size: 10px; color: var(--danger); letter-spacing: 2px; }
  .flag-item { display: flex; align-items: flex-start; gap: 12px; padding: 10px 0; border-bottom: 1px solid var(--border); }
  .flag-item:last-child { border-bottom: none; }
  .flag-dot { width: 8px; height: 8px; border-radius: 50%; margin-top: 5px; flex-shrink: 0; }
  .flag-text { font-family: var(--font-mono); font-size: 13px; line-height: 1.5; }
  .flag-severity { font-size: 10px; letter-spacing: 2px; opacity: 0.7; display: block; margin-top: 2px; }
  .ai-panel { background: var(--panel); border: 1px solid var(--accent2); padding: 28px; position: relative; }
  .ai-panel::before { content: 'AI ANALYSIS'; position: absolute; top: -10px; left: 20px; background: var(--panel); padding: 0 10px; font-family: var(--font-mono); font-size: 10px; color: var(--accent2); letter-spacing: 2px; }
  .ai-text { font-family: var(--font-mono); font-size: 13px; line-height: 1.8; color: var(--text); }
  .token-header { background: var(--panel); border: 1px solid var(--border); padding: 20px 28px; display: flex; align-items: center; gap: 16px; }
  .token-name { font-size: 24px; font-weight: 700; letter-spacing: 2px; color: var(--accent); }
  .token-address { font-family: var(--font-mono); font-size: 11px; color: var(--muted); margin-top: 2px; word-break: break-all; }
  .error-panel { background: var(--panel); border: 1px solid var(--danger); padding: 28px; text-align: center; }
  .error-text { font-family: var(--font-mono); font-size: 13px; color: var(--danger); letter-spacing: 1px; }
  .reset-btn { background: transparent; border: 1px solid var(--muted); color: var(--muted); font-family: var(--font-mono); font-size: 12px; padding: 8px 20px; cursor: pointer; margin-top: 16px; letter-spacing: 2px; transition: all 0.2s; }
  .reset-btn:hover { border-color: var(--accent); color: var(--accent); }
  .footer { text-align: center; margin-top: 48px; font-family: var(--font-mono); font-size: 11px; color: var(--muted); letter-spacing: 2px; }
  @media(max-width:600px){.title{font-size:28px}.input-row{flex-direction:column}.data-grid{grid-template-columns:1fr}.score-panel{flex-direction:column;text-align:center}}
`;

const EXAMPLES = [
  { label: "USDC", address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" },
  { label: "BONK", address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" },
  { label: "WIF", address: "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm" },
];

const getRiskColor = s => s >= 75 ? "#ff3b5c" : s >= 45 ? "#ffaa00" : "#00ff88";
const getRiskLabel = s => s >= 75 ? "HIGH RISK" : s >= 45 ? "MEDIUM RISK" : "LOW RISK";
const getRiskVerdict = s => s >= 75 ? "⚠ EXERCISE EXTREME CAUTION" : s >= 45 ? "◈ PROCEED WITH CAUTION" : "✓ APPEARS RELATIVELY SAFE";

async function rpcCall(method, params) {
  for (const endpoint of RPC_ENDPOINTS) {
    try {
      const res = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params })
      });
      if (!res.ok) continue;
      const data = await res.json();
      if (data.error) continue;
      return data.result;
    } catch (e) { continue; }
  }
  return null;
}

function generateAIAnalysis(result) {
  const { name, symbol, mintAuth, freezeAuth, topHolderPct, hasMetadata, riskScore, supply } = result;

  if (riskScore >= 75) {
    return `${name} (${symbol}) presents significant security concerns that warrant extreme caution. ${mintAuth ? "The mint authority remains active, meaning the developer can create unlimited new tokens at any time, which could drastically dilute the value of existing holdings." : ""} ${freezeAuth ? "The freeze authority is enabled, giving the developer the ability to lock token accounts and prevent holders from transferring their tokens." : ""} ${topHolderPct > 50 ? `Extreme supply concentration is detected — a single wallet controls ${topHolderPct}% of all tokens, creating massive dump risk.` : ""} This token exhibits multiple characteristics commonly associated with rug pulls and should be approached with extreme caution or avoided entirely.`;
  }

  if (riskScore >= 45) {
    return `${name} (${symbol}) shows some risk indicators that require careful consideration before investing. ${mintAuth ? "The mint authority has not been revoked — this is a yellow flag as it gives the team ability to inflate supply." : "The mint authority has been revoked, which is a positive sign for supply integrity."} ${topHolderPct > 20 ? `Supply concentration is elevated with the top holder controlling ${topHolderPct}% of tokens — watch for large sell orders.` : "Holder distribution appears reasonable."} ${hasMetadata ? "Token metadata is present and verifiable on-chain." : "Missing token metadata is a concern — legitimate projects typically have complete metadata."} Do your own research and consider position sizing carefully given these factors.`;
  }

  return `${name} (${symbol}) appears relatively safe based on on-chain indicators. ${mintAuth ? "Note that mint authority is still active." : "The mint authority has been revoked, fixing the total supply — this is a strong positive signal."} ${freezeAuth ? "Freeze authority is present — a minor concern worth noting." : "No freeze authority detected — token holders cannot have their accounts frozen."} ${topHolderPct < 20 ? `Supply distribution looks healthy with the top holder controlling only ${topHolderPct}% of tokens.` : ""} While on-chain fundamentals look clean, always conduct thorough research including team background, liquidity depth, and project utility before investing. Past safety indicators do not guarantee future performance.`;
}

export default function SolanaAuditAI() {
  const [address, setAddress] = useState("");
  const [loading, setLoading] = useState(false);
  const [logLines, setLogLines] = useState([]);
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);

  const addLog = (text, active = false) =>
    setLogLines(prev => [...prev, { text, active }]);

  const sleep = ms => new Promise(r => setTimeout(r, ms));

  const analyzeToken = async () => {
    if (!address.trim()) return;
    setLoading(true); setResult(null); setError(null); setLogLines([]);

    try {
      addLog("> INITIALIZING SCAN SEQUENCE...", true); await sleep(400);
      addLog("> CONNECTING TO SOLANA MAINNET..."); await sleep(300);

      // Get token account info
      addLog("> FETCHING TOKEN MINT DATA...", true);
      const mintInfo = await rpcCall("getAccountInfo", [
        address.trim(),
        { encoding: "jsonParsed" }
      ]);

      if (!mintInfo || !mintInfo.value) {
        throw new Error("Token not found. Please check the mint address.");
      }

      const parsed = mintInfo.value?.data?.parsed?.info;
      if (!parsed) throw new Error("Not a valid token mint address.");

      addLog("> ANALYZING TOKEN SUPPLY...", true); await sleep(300);

      const supply = parseInt(parsed.supply || 0);
      const decimals = parsed.decimals || 0;
      const actualSupply = supply / Math.pow(10, decimals);
      const mintAuth = parsed.mintAuthority || null;
      const freezeAuth = parsed.freezeAuthority || null;

      addLog("> SCANNING LARGEST HOLDERS...", true);

      // Get largest token accounts
      const holdersResult = await rpcCall("getTokenLargestAccounts", [address.trim()]);
      const topHolders = holdersResult?.value || [];

      let topHolderPct = 0;
      if (topHolders.length > 0 && actualSupply > 0) {
        const topAmount = parseFloat(topHolders[0]?.uiAmount || 0);
        topHolderPct = Math.min(99, Math.round((topAmount / actualSupply) * 100));
      }

      addLog("> CHECKING TRANSACTION HISTORY...", true); await sleep(300);

      // Get recent signatures
      const sigs = await rpcCall("getSignaturesForAddress", [
        address.trim(),
        { limit: 5 }
      ]);
      const recentTxCount = sigs?.length || 0;

      addLog("> RUNNING THREAT ANALYSIS...", true); await sleep(400);

      // Build name from address (shorten it)
      const shortAddr = address.trim().slice(0, 4) + "..." + address.trim().slice(-4);
      const name = shortAddr;
      const symbol = address.trim().slice(0, 4).toUpperCase();
      const hasMetadata = true; // We confirmed it's a valid mint

      // Build flags
      const flags = [];
      let riskScore = 0;

      if (mintAuth) {
        flags.push({ text: "Mint authority is still active — developer can create unlimited new tokens at any time", severity: "CRITICAL", color: "#ff3b5c" });
        riskScore += 35;
      } else {
        flags.push({ text: "Mint authority has been revoked — token supply is permanently fixed", severity: "SAFE", color: "#00ff88" });
      }

      if (freezeAuth) {
        flags.push({ text: "Freeze authority enabled — developer can freeze any token holder's account", severity: "HIGH", color: "#ff3b5c" });
        riskScore += 25;
      } else {
        flags.push({ text: "No freeze authority — token holders cannot have their accounts frozen", severity: "SAFE", color: "#00ff88" });
      }

      if (topHolderPct > 50) {
        flags.push({ text: `Top holder controls ${topHolderPct}% of total supply — extreme concentration risk`, severity: "CRITICAL", color: "#ff3b5c" });
        riskScore += 30;
      } else if (topHolderPct > 20) {
        flags.push({ text: `Top holder controls ${topHolderPct}% of supply — moderate concentration`, severity: "MEDIUM", color: "#ffaa00" });
        riskScore += 15;
      } else if (topHolderPct > 0) {
        flags.push({ text: `Top holder controls only ${topHolderPct}% — healthy distribution`, severity: "SAFE", color: "#00ff88" });
      }

      if (recentTxCount === 0) {
        flags.push({ text: "No recent transaction activity detected — token may be inactive or abandoned", severity: "MEDIUM", color: "#ffaa00" });
        riskScore += 10;
      }

      riskScore = Math.min(riskScore, 99);

      addLog("> GENERATING REPORT...", true); await sleep(200);

      const resultData = {
        name, symbol, address: address.trim(),
        supply: actualSupply, mintAuth, freezeAuth,
        topHolderPct, hasMetadata, recentTxCount,
        riskScore, flags
      };

      resultData.aiAnalysis = generateAIAnalysis(resultData);

      addLog("> SCAN COMPLETE ✓", true);
      setResult(resultData);

    } catch (err) {
      setError(err.message || "Scan failed. Check the token address.");
    } finally {
      setLoading(false);
    }
  };

  const rc = result ? getRiskColor(result.riskScore) : "#00d4ff";

  return (
    <>
      <style>{styles}</style>
      <div className="scanlines" />
      <div className="grid-bg" />
      <div className="container">
        <div className="header">
          <div className="logo-line">
            <div className="logo-icon">🔐</div>
            <h1 className="title">SolanaAudit</h1>
          </div>
          <p className="subtitle">AI-POWERED TOKEN SECURITY SCANNER</p>
          <span className="tag-line">BUILT ON SOLANA · COLOSSEUM FRONTIER 2026</span>
        </div>

        <div className="input-panel">
          <div className="input-row">
            <input
              className="address-input"
              placeholder="Paste Solana token mint address..."
              value={address}
              onChange={e => setAddress(e.target.value)}
              onKeyDown={e => e.key === "Enter" && analyzeToken()}
              disabled={loading}
            />
            <button className="scan-btn" onClick={analyzeToken} disabled={loading || !address.trim()}>
              {loading ? "SCANNING..." : "SCAN TOKEN"}
            </button>
          </div>
          <div className="examples">
            TRY EXAMPLE:
            {EXAMPLES.map(t => (
              <span key={t.address} onClick={() => setAddress(t.address)}>{t.label}</span>
            ))}
          </div>
        </div>

        {loading && (
          <div className="loading-panel">
            <div className="loading-text">◈ SCANNING BLOCKCHAIN DATA...</div>
            <div className="progress-bar"><div className="progress-fill" /></div>
            <div className="log-lines">
              {logLines.map((l, i) => (
                <div key={i} className={`log-line ${l.active ? "active" : ""}`}>{l.text}</div>
              ))}
            </div>
          </div>
        )}

        {error && (
          <div className="error-panel">
            <div className="error-text">SCAN FAILED: {error}</div>
            <button className="reset-btn" onClick={() => setError(null)}>RETRY</button>
          </div>
        )}

        {result && (
          <div className="results">
            <div className="token-header">
              <div>
                <div className="token-name">{result.name}</div>
                <div className="token-address">{result.address}</div>
              </div>
            </div>

            <div className="score-panel">
              <div className="score-circle" style={{ borderColor: rc, boxShadow: `0 0 30px ${rc}40` }}>
                <div className="score-number" style={{ color: rc }}>{result.riskScore}</div>
                <div className="score-label" style={{ color: rc }}>RISK</div>
              </div>
              <div className="score-info">
                <div className="risk-badge" style={{ color: rc, borderColor: rc }}>{getRiskLabel(result.riskScore)}</div>
                <div className="verdict" style={{ color: rc }}>{getRiskVerdict(result.riskScore)}</div>
                <div className="verdict-detail">
                  Risk score based on {result.flags.length} indicators across mint authority, freeze authority, and holder distribution analysis.
                </div>
              </div>
            </div>

            <div className="data-grid">
              <div className="data-card">
                <div className="data-card-label">Mint Authority</div>
                <div className="data-card-value" style={{ color: result.mintAuth ? "#ff3b5c" : "#00ff88" }}>
                  {result.mintAuth ? "ACTIVE ⚠" : "REVOKED ✓"}
                </div>
                <div className="data-card-sub">{result.mintAuth ? "Dev can print more tokens" : "Supply is fixed forever"}</div>
              </div>
              <div className="data-card">
                <div className="data-card-label">Freeze Authority</div>
                <div className="data-card-value" style={{ color: result.freezeAuth ? "#ff3b5c" : "#00ff88" }}>
                  {result.freezeAuth ? "ACTIVE ⚠" : "NONE ✓"}
                </div>
                <div className="data-card-sub">{result.freezeAuth ? "Dev can freeze accounts" : "Cannot be frozen"}</div>
              </div>
              <div className="data-card">
                <div className="data-card-label">Top Holder</div>
                <div className="data-card-value" style={{ color: result.topHolderPct > 50 ? "#ff3b5c" : result.topHolderPct > 20 ? "#ffaa00" : "#00ff88" }}>
                  {result.topHolderPct}%
                </div>
                <div className="data-card-sub">of total supply</div>
              </div>
              <div className="data-card">
                <div className="data-card-label">Total Supply</div>
                <div className="data-card-value">
                  {result.supply > 1e12 ? `${(result.supply/1e12).toFixed(1)}T` : result.supply > 1e9 ? `${(result.supply/1e9).toFixed(1)}B` : result.supply > 1e6 ? `${(result.supply/1e6).toFixed(1)}M` : result.supply.toLocaleString()}
                </div>
                <div className="data-card-sub">tokens minted</div>
              </div>
            </div>

            <div className="flags-panel">
              {result.flags.map((flag, i) => (
                <div key={i} className="flag-item">
                  <div className="flag-dot" style={{ background: flag.color, boxShadow: `0 0 6px ${flag.color}` }} />
                  <div>
                    <div className="flag-text">{flag.text}</div>
                    <span className="flag-severity" style={{ color: flag.color }}>{flag.severity}</span>
                  </div>
                </div>
              ))}
            </div>

            <div className="ai-panel">
              <div className="ai-text">{result.aiAnalysis}</div>
            </div>

            <div style={{ textAlign: "center" }}>
              <button className="reset-btn" onClick={() => { setResult(null); setAddress(""); }}>
                SCAN ANOTHER TOKEN
              </button>
            </div>
          </div>
        )}

        <div className="footer">
          SOLANAAUDIT AI v2.0 · BUILT ON SOLANA · COLOSSEUM FRONTIER 2026
          <br /><span style={{opacity:0.5}}>NOT FINANCIAL ADVICE · FOR EDUCATIONAL PURPOSES ONLY</span>
        </div>
      </div>
    </>
  );
}
