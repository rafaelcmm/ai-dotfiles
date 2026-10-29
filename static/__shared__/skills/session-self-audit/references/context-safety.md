# Context safety for session transcripts

One transcript record can exceed a megabyte or embed a whole history.
Printing one whole record can overflow the context of the session doing
the audit. Every reader of a session file — the dispatching agent or an
analyst subagent — follows these rules for every file, every time.

1. **Measure before reading.**

   ```bash
   wc -lc "$F"
   awk '{ if (length($0) > 100000) print NR, length($0) }' "$F"   # long lines
   ```

2. **Never `cat` or `grep` for content.** Get line numbers and counts
   first (`grep -n … | cut -d: -f1`, `jq -r '.type' | sort | uniq -c`),
   then pull small fields from specific lines (`sed -n Np | jq -c '{…}'`
   or `| cut -c1-500`). Use the field-extraction commands or queries
   established for the source in front of you — do not assume another
   harness's record shape.
3. **Narrow anything over 500 characters.** If a command returns more
   than 500 characters for one record, tighten the field or the slice
   before reading it.
4. **Read-only.** Never modify, move, or delete a session file.
