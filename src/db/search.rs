//! Cross-entity full-text search backed by SQLite FTS5.
//!
//! The `search_fts` virtual table is populated by triggers in
//! [`crate::db::schema`]. Each row carries a `kind` discriminator
//! (issue / pull / triage / comment) so the same query can return hits
//! from any source. Results are ordered by FTS5's BM25 rank — most
//! relevant first.
//!
//! Per-repo storage means the search handler in `wshm-pro` iterates
//! every configured repo's `Database` and merges results.

use anyhow::Result;
use rusqlite::params;
use serde::Serialize;

use crate::db::Database;

#[derive(Clone, Debug, Serialize)]
pub struct SearchHit {
    /// One of `"issue"`, `"pull"`, `"triage"`, `"comment"`.
    pub kind: String,
    /// Issue / PR number (for `comment`, the parent issue's number).
    pub number: u64,
    /// Issue or PR title. Empty for `comment` rows.
    pub title: String,
    /// Short snippet around the matched terms (FTS5 `snippet()`).
    pub snippet: String,
    /// Updated/acted/created timestamp depending on kind.
    pub updated_at: String,
    /// BM25 rank — lower is better; included for caller-side sort
    /// merges across repos.
    pub rank: f64,
}

impl Database {
    /// Run an FTS5 search over the local repo's `search_fts` table.
    ///
    /// `query` is interpreted as an FTS5 MATCH expression. To search
    /// for a literal phrase, the caller should wrap it in double
    /// quotes (`"foo bar"`). The handler in wshm-pro takes care of
    /// escaping user input.
    ///
    /// Returns at most `limit` hits ordered by rank ascending (best
    /// first). `limit` is clamped to [1, 500] internally to bound
    /// per-repo cost — the caller paginates the merged result set.
    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let limit = limit.clamp(1, 500);
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT kind, number, title,
                        snippet(search_fts, -1, '<mark>', '</mark>', '…', 16) AS snip,
                        updated_at,
                        bm25(search_fts) AS rank
                 FROM search_fts
                 WHERE search_fts MATCH ?1
                 ORDER BY rank ASC
                 LIMIT ?2",
            )?;
            let rows = stmt
                .query_map(params![query, limit as i64], |r| {
                    let number: i64 = r.get(1)?;
                    Ok(SearchHit {
                        kind: r.get(0)?,
                        number: number as u64,
                        title: r.get::<_, String>(2)?,
                        snippet: r.get::<_, String>(3)?,
                        updated_at: r.get(4)?,
                        rank: r.get(5)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
    }
}

/// Sanitize a free-form user query into a safe FTS5 MATCH expression.
///
/// FTS5 syntax includes operators (`AND`, `OR`, `NOT`, `NEAR`, parens,
/// double quotes) which a naive user would not expect to be parsed.
/// We escape the input by:
///   1. Stripping characters that have meaning in FTS5: `"()*'`.
///   2. Splitting on whitespace and treating each token as a prefix
///      match (`token*`), so `hermes` matches `hermes-agent` etc.
///   3. ANDing tokens together — typed-ahead UX expects narrowing.
///
/// Empty / whitespace-only input returns `None` so the caller can
/// short-circuit.
pub fn sanitize_query(input: &str) -> Option<String> {
    let cleaned: String = input
        .chars()
        .map(|c| match c {
            '"' | '(' | ')' | '*' | '\'' | ':' => ' ',
            other => other,
        })
        .collect();
    let tokens: Vec<String> = cleaned
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| format!("{t}*"))
        .collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" AND "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn sanitize_basic() {
        assert_eq!(sanitize_query("hermes").as_deref(), Some("hermes*"));
        assert_eq!(
            sanitize_query("hermes agent").as_deref(),
            Some("hermes* AND agent*")
        );
        assert_eq!(sanitize_query("   ").as_deref(), None);
    }

    #[test]
    fn sanitize_strips_fts_operators() {
        // Operators must not survive into the MATCH expression.
        let q = sanitize_query("a\"b c(d) NOT*x").unwrap();
        assert!(!q.contains('"'));
        assert!(!q.contains('('));
        assert!(!q.contains(')'));
        // Each token still becomes a prefix match.
        assert!(q.contains("a*"));
    }

    #[test]
    fn search_fts_finds_issue_title() -> Result<()> {
        let db = Database::open_memory()?;
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO issues (number, title, body, state, labels, author,
                                     created_at, updated_at, reactions_plus1, reactions_total)
                 VALUES (42, 'Add hermes agent support', 'Could you wire it like Claude Code?',
                         'open', '[]', 'someone', '2026-05-06T12:00:00Z',
                         '2026-05-06T12:00:00Z', 0, 0)",
                [],
            )?;
            Ok(())
        })?;

        let q = sanitize_query("hermes").unwrap();
        let hits = db.search_fts(&q, 10)?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "issue");
        assert_eq!(hits[0].number, 42);
        assert!(hits[0].snippet.contains("hermes") || hits[0].snippet.contains("<mark>"));
        Ok(())
    }
}
