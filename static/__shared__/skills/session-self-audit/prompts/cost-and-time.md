Read `prompts/analyst-common.md` first; it gives your role, inputs,
context-safety rules, and the return format. This file adds the dimension.

Dimension: Cost and time

Account for where tokens and wall-clock went.

1. Tokens. Use only the usage records and counter meanings your dispatcher
   gave you. State whether each counter is incremental or cumulative
   *before* totalling — differencing a cumulative counter and summing an
   incremental one give different answers, and guessing wrong silently
   inflates or deflates every total downstream. Report the five turns with
   the largest supported totals and the supported totals per associated
   session.
2. Wall-clock. Use the evidenced timestamp fields and event boundaries your
   dispatcher gave you. Report the five longest supported turns and any gap
   longer than ten minutes between consecutive events (idle, waiting on an
   associated session, or waiting on the user — say which only when the
   records show it).
3. Largest tool results: report the ten largest results with their tool and
   turn. Measure records (`wc -c`, length checks) before extracting any
   bounded content from them.
4. Compactions: count and locate records whose meaning as compaction events
   your dispatcher established. Report available before/after counters and
   what the session was doing when each fired; mark unsupported fields
   absent rather than guessing.
5. Associated sessions (subagents): count them and report supported usage,
   duration, and dispatching turn for each.
6. Name the turns, subagents, tools, or repeated calls that dominate the
   totals, with numbers attached. Do not speculate about why a turn was
   expensive beyond what the transcript shows.
