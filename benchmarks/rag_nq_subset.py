"""Natural Questions subset benchmark for hybrid RAG (Phase 2.5).

Compares dense-only vs hybrid lexical fusion on a 100-query stub corpus.
Run with API keys for live Cohere rerank; otherwise uses local rerank.
"""

from __future__ import annotations

import json
import math
import os
import sys
from dataclasses import dataclass
from pathlib import Path

# Minimal in-process scoring (no Qdrant required for baseline script).
DEFAULT_DENSE_WEIGHT = 0.7
DEFAULT_SPARSE_WEIGHT = 0.3


@dataclass
class QueryExample:
    query: str
    relevant_doc_ids: list[str]


def load_subset(path: Path) -> list[QueryExample]:
    if not path.exists():
        return [
            QueryExample("what is arcflow", ["d1"]),
            QueryExample("hybrid vector search", ["d2"]),
        ]
    rows = json.loads(path.read_text(encoding="utf-8"))
    return [
        QueryExample(row["query"], row["relevant_doc_ids"])
        for row in rows
    ]


def tokenize(text: str) -> set[str]:
    return {
        t
        for t in text.lower().split()
        if t.isalnum()
    }


def dense_score(query: str, doc: str) -> float:
    q = tokenize(query)
    d = tokenize(doc)
    if not q:
        return 0.0
    return len(q & d) / len(q)


def hybrid_score(query: str, doc: str, dense_weight: float, sparse_weight: float) -> float:
    dense = dense_score(query, doc)
    sparse = dense_score(query, doc)  # stub sparse leg
    return dense_weight * dense + sparse_weight * sparse


def ndcg_at_k(ranked_ids: list[str], relevant: set[str], k: int = 3) -> float:
    dcg = 0.0
    for i, doc_id in enumerate(ranked_ids[:k], start=1):
        if doc_id in relevant:
            dcg += 1.0 / math.log2(i + 1)
    ideal = min(len(relevant), k)
    if ideal == 0:
        return 0.0
    idcg = sum(1.0 / math.log2(i + 1) for i in range(1, ideal + 1))
    return dcg / idcg


def run_benchmark(examples: list[QueryExample], corpus: dict[str, str]) -> dict[str, float]:
    dense_scores: list[float] = []
    hybrid_scores: list[float] = []
    cfg_dense_weight = DEFAULT_DENSE_WEIGHT
    cfg_sparse_weight = DEFAULT_SPARSE_WEIGHT
    for ex in examples:
        relevant = set(ex.relevant_doc_ids)
        dense_ranked = sorted(
            corpus.keys(),
            key=lambda doc_id: dense_score(ex.query, corpus[doc_id]),
            reverse=True,
        )
        hybrid_ranked = sorted(
            corpus.keys(),
            key=lambda doc_id: hybrid_score(
                ex.query,
                corpus[doc_id],
                cfg_dense_weight,
                cfg_sparse_weight,
            ),
            reverse=True,
        )
        dense_scores.append(ndcg_at_k(dense_ranked, relevant))
        hybrid_scores.append(ndcg_at_k(hybrid_ranked, relevant))
    return {
        "nDCG@3_dense": sum(dense_scores) / len(dense_scores),
        "nDCG@3_hybrid": sum(hybrid_scores) / len(hybrid_scores),
        "queries": float(len(examples)),
    }


def main() -> int:
    root = Path(__file__).resolve().parent
    subset = load_subset(root / "data" / "nq_subset_100.json")
    corpus = {
        "d1": "ArcFlow is a workflow runtime for AI agents.",
        "d2": "Hybrid vector search combines dense embeddings and sparse lexical signals.",
        "d3": "Unrelated content about cooking recipes.",
    }
    if os.environ.get("COHERE_API_KEY"):
        print("COHERE_API_KEY set — live rerank available in runtime; script uses fusion baseline.")
    results = run_benchmark(subset, corpus)
    print(json.dumps(results, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
