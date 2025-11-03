# scrapster

Rust-powered system metrics collector exposed to Python via PyO3.

## Installation

```
pip install scrapster
```

## Usage

```python
import scrapster

m = scrapster.get_metrics_once(1000)  # wait ~1s and return a dict
print(m["cpu_usage_percent"], m["mem_used_bytes"])
```

## Notes
- Some metrics rely on Linux `/proc/*` files; on non-Linux systems these may be zero.

