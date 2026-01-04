
from __future__ import annotations
import random
import math
from typing import Dict, Tuple

def gen_dist(n: int) -> Dict[str, Tuple[float, Tuple[float, float]]]:
    # Dirichlet-like generation
    xs = [random.random() for _ in range(n)]
    s = sum(xs) or 1.0
    probs = [x / s for x in xs]
    out = {}
    for i, p in enumerate(probs):
        # create a CI around p
        width = random.uniform(0.0, 0.2)
        lo = max(0.0, p - width)
        hi = min(1.0, p + width)
        out[f"o{i}"] = (p, (lo, hi))
    return out

def check_invariants(out: Dict[str, Tuple[float, Tuple[float, float]]]) -> None:
    total = 0.0
    for _, (p, (lo, hi)) in out.items():
        assert 0.0 <= p <= 1.0
        assert 0.0 <= lo <= 1.0
        assert 0.0 <= hi <= 1.0
        assert lo <= p <= hi
        total += p
    assert abs(total - 1.0) < 1e-6 or (0.999 <= total <= 1.001)

def main() -> None:
    iters = int(random.choice([1000, 2000, 5000]))
    for _ in range(iters):
        n = random.randint(2, 10)
        out = gen_dist(n)
        check_invariants(out)
    print(f"ok: {iters} cases")

if __name__ == "__main__":
    main()
