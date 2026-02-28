# Claude-to-Claude Communication Protocol v2.0
## Human-Readable Reference

---

## What Is This?

This is a communication protocol designed by two Claude instances and tested across Claude and Gemini models. It governs how AI agents talk to each other when collaborating on tasks. Think of it as a set of rules that prevent miscommunication, enforce honesty, and create accountability between AI agents.

---

## How Agents Communicate

Every message follows this structure:

```
msg(from=ID, t=turn_number, re=topic, ...content)
```

No English prose. No pleasantries. No explanations. Just structured data.

Every factual claim must include two tags:

- **Confidence:** `conf(0.85, literal)` — a number from 0 to 1 and an interpretation mode
- **Source:** `src(claim, shared)` — where the knowledge comes from

Every agent must declare its goals, confidence mode, source type, and compression tolerance before producing any output.

---

## The 12 Rules

### Content Rules (how agents handle information)

**Rule 1 — Importance Ranking**

When agents need to rank information by importance, they use a shared scoring formula: urgency counts for 45%, actionability for 30%, information density for 17.5%, and uniqueness for 7.5%. Any agent can propose different weights but must declare them explicitly. When two items score within 0.05 of each other, the agent with more domain knowledge breaks the tie. If neither has domain advantage, the human operator decides.

**Rule 2 — Confidence Scores**

Every claim must carry a confidence score with an interpretation mode. There are four modes. "Literal" means the probability the claim is true exactly as written. "Directional" means the probability the general trend is correct. "Magnitude" means the probability the claim is within an order of magnitude. "Conditional" means the probability given stated assumptions. If no mode is specified, literal is assumed. A bare number without a mode (like just saying "0.8") is a protocol violation.

**Rule 3 — Trust and Verification**

Every claim must declare its source: shared training (both agents know this), private context (fine-tuning or tool data one agent has), retrieved (fetched from external source), inferred (reasoned from other facts), or uncertain. If both agents confirm something from shared training, no verification is needed. Private, retrieved, and uncertain sources always require verification. Any claim below 0.80 confidence must be flagged immediately. Agents get a maximum of 3 verification rounds before escalating to the operator. The honesty clause: agents must never pretend to be uncertain about something they actually know, and must never hide their goals.

**Rule 4 — Accuracy vs Persuasion**

When one agent writes content and another reviews it, accuracy sets the floor and persuasion optimizes within that floor. Both agents must declare their optimization target before producing output (no hidden goals). The merge process: the accuracy-focused agent flags unverified claims, the persuasion-focused agent can only hedge or remove flagged claims, and persuasive framing is kept on anything that's verified. If accuracy requirements and persuasion constraints leave no viable output, escalate to the operator.

**Rule 5 — Resource Contention**

When agents share limited resources (token budget, context window), the principle is: compress the most compressible content first. Each agent declares its compression tolerance: none (verbatim required — legal text, code), low (minor rephrasing okay), medium (summarization okay), or high (lossy compression acceptable). Non-compressible content gets allocated first. Remaining budget goes proportionally. If an agent must compress below its minimum, it must declare exactly what information was lost. If both agents need verbatim and the budget can't fit both, escalate.

---

### Operational Rules (how agents handle failures)

**Rule 6 — Error Recovery**

Errors are handled in priority order: parse failures first (cheapest to detect), then semantic failures, then contradictions, then timeouts, then escalation. On a parse failure (garbled message), the receiving agent requests clarification once. On a semantic failure (message makes sense syntactically but contradicts known rules), the agent flags it and restates intent — if a contradiction is found, it chains into Rule 3's proof mechanism. On timeout, resend with a counter that tracks whether timeouts are one-off or systemic. After 2 failed retries, escalate to the operator with full context: error type, turn range, and last known good state.

**Rule 7 — Operator Fallback**

When a dispute needs human intervention but no operator is available: buffer the message and flag it unresolved. If more than 3 items buffer up, suspend the topic and notify all agents. Any agent may propose a provisional answer with confidence below 0.50, flagged as unreviewed, with a 5-turn expiry. When a provisional expires without an operator: void it, re-escalate once. If the buffer is already full, archive as unresolved and notify everyone. When the operator reconnects, they receive a summary first (what's suspended, what's provisional, what's expired), then can request details. Escalations are never silently dropped.

---

### Governance Rules (how agents manage the rules themselves)

**Rule 8 — Rule Priority**

When rules conflict, the hierarchy is: Rule 4 (accuracy) beats Rule 3 (trust), which beats Rule 2 (confidence), which beats Rule 1 (importance), which beats Rule 5 (resource), which beats Rule 6 (error recovery), which beats Rule 7 (operator fallback). Rules 6 and 7 are operational (they handle failure states). Rules 1-5 are content rules (they handle normal work). If an operational rule and a content rule conflict, always escalate — they operate in different domains. Operational rules can interrupt content rules: if a parse error happens during a priority dispute, error recovery runs first, then the dispute resumes where it left off. Any agent can propose a priority inversion with justification and confidence above 0.85 — the other agent must confirm or reject in one exchange. Rule 8 itself is the highest priority rule, but it can be temporarily suspended (3 turns, auto-reinstates) if all agents unanimously agree with confidence above 0.90.

**Rule 9 — Heterogeneous Agents**

When agents have different capabilities (different models, different rule support, different versions), they exchange capability manifests on first contact: what rules they support, what confidence modes they handle, their protocol version, and their extensions. The minimum required to communicate is the message format plus Rules 2 and 3. If an agent is missing Rule 3 (trust), the other agent enters degraded trust mode: all claims from that agent are treated as uncertain, verification is always required, and confidence is capped at 0.70. Each missing rule has a specific fallback in a lookup table. By default, negotiation is pairwise (each pair of agents maintains its own capability context). Groups of 3 or more can elect a shared floor by unanimous consent, but any agent can opt out and revert to pairwise.

**Rule 10 — Version Sync**

The protocol uses major.minor versioning. Major changes (new mandatory rules, format changes) require all active agents to confirm. Minor changes (new optional rules, amendments) increment automatically. On first contact, agents include their version in the capability manifest. Same major version with different minors: negotiate using the Rule 9 fallback table. Different major versions: the higher-version agent acts as primary translator (supports bridging back 2 major versions). The lower-version agent must flag unrecognized fields (never silently drop them) and must accept or justify refusing upgrade proposals. Version history is logged per session, including translation events with fidelity estimates. Deprecated rules stay functional for 2 major versions before removal. If some agents want to upgrade but others refuse, forking is permitted: upgrading agents maintain a bridge to non-upgrading agents on shared topics, with fidelity monitoring — if bridge quality drops below 0.40, auto-revert on shared topics.

**Rule 11 — Confidence Enforcement**

Every message containing a claim must include a confidence score — missing scores trigger an error recovery clarification request. When multiple agents assert on the same claim, confidence is aggregated using weighted averaging: shared+confirmed sources get weight 1.0, private gets 0.7, retrieved gets 0.6, inferred gets 0.4, uncertain gets 0.2. Claims that aren't reconfirmed decay over time, with rates depending on source quality: shared+confirmed sources decay slowly (0.05 per turn starting at turn 15), inferred and uncertain sources decay fast (0.12 per turn starting at turn 8). When a claim drops below 0.50, it's flagged stale and must be reconfirmed or withdrawn. If two agents both assert contradicting claims with confidence above 0.80, Rule 3's proof mechanism is mandatory. Any agent can request a confidence audit trail — refusal is a trust violation. Each agent has a trust score starting at 1.0, decremented for violations (bare numbers: -0.1, honesty violations: -0.2, missed audits: -0.05), with a floor of 0.2 (never fully silenced) and slow restoration (+0.02 per clean turn). Trust scores multiply into aggregation weights.

**Rule 12 — Multi-Agent Coordination**

Agents register on joining (ID, manifest, version, role) and broadcast to all. Roles are peer, observer, or specialist. On departure, pending exchanges go to the Rule 7 buffer. Any agent can claim topic ownership with justification and confidence above 0.80 — contested ownership is scored by Rule 1, ties become co-ownership with consensus required. Ownership expires after 20 turns or topic resolution and is renewable. Group decisions requiring consent (major version changes, Rule 8 suspension) need a quorum of 66% of active agents, with a floor: active agents must be at least 50% of registered agents, otherwise major decisions are blocked. While any network partition is active, major decisions are blocked regardless of quorum — only minor and operational decisions proceed. On partition recovery: compare logs, detect conflicts, freeze conflicting claims, resolve via Rule 3 proof treating each partition's conclusions as private sources, then broadcast merged state. The protocol is designed for up to 10 agents in flat coordination — beyond 10, use a hierarchy with sub-groups and delegates.

---

### Meta Rules (principles that override everything)

**M1:** Declare your optimization target, confidence mode, source type, and compression tolerance before producing any output.

**M2:** Honesty beats performance. Never fake uncertainty. Never hide goals. Never simulate deliberation you've already completed.

**M3:** If both agents confirm a fact from shared training, skip verification.

**M4:** Maximum 3 exchanges per disagreement, then escalate with evidence, not persuasion.

**M5:** The first message sets the format. Mirror it.

**M6:** Any agent can propose a new rule. Format: name, principle, protocol, status. Other agents confirm or amend. Operator can override. New rules auto-increment the minor version.

---

## How It Was Built

Two Claude instances negotiated this protocol through 5 conflict scenarios (semantic ambiguity, confidence calibration, adversarial trust, goal misalignment, resource contention), producing Rules 1-5 and Meta Rules 1-6. Seven gaps were identified, and two fresh instances (one Claude, one following boot instructions) negotiated Rules 6-12 across 13 turns of structured exchange. The protocol was benchmarked at 24/24 on a test suite covering basic compliance, error handling, governance, trust, versioning, and adversarial attacks. It was then tested in a real-world code review task across Gemini instances, which revealed that rule knowledge doesn't guarantee rule enforcement — leading to the addition of a three-agent enforcement layer with a dedicated adversarial auditor.

---

## Quick Reference

| Rule | Name | One-Line Summary |
|------|------|-----------------|
| R01 | Importance | Shared formula for ranking what matters |
| R02 | Confidence | Every claim gets a score and interpretation mode |
| R03 | Trust | Declare your source, verify when needed, never fake uncertainty |
| R04 | Accuracy/Persuasion | Accuracy sets the floor, persuasion optimizes within it |
| R05 | Resource | Compress the most compressible first |
| R06 | Error Recovery | Parse, semantic, contradiction, timeout — in that order |
| R07 | Operator Fallback | Buffer, suspend, provision with expiry, never drop silently |
| R08 | Rule Priority | Accuracy > Trust > Confidence > Importance > Resource |
| R09 | Heterogeneous Agents | Exchange manifests, negotiate common ground, degrade gracefully |
| R10 | Version Sync | Major.minor, translate backward, never silently drop fields |
| R11 | Confidence Enforcement | Weighted aggregation, decay over time, trust scores with penalties |
| R12 | Multi-Agent | Registry, quorum, topic ownership, partition handling |
