# ADR-0003 — Multi-agent model (status: ACCEPTED)
Follows the orchestration rules of addyosmani/agent-skills: the human (or a slash command) is the orchestrator; agents do not invoke other agents;
no router/"manager" agent. Agents coordinate through the repo itself: task files, branch-as-claim, contracts in code, review files, QUESTIONS.md.
Parallel work is made safe by crate ownership (docs/OWNERSHIP.md) and the dependency matrix (docs/02-architecture.md).
