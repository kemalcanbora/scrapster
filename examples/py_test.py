import time

import scrapster


def main() -> None:
    # Collect 3 samples, 1s apart
    for i in range(3):
        m = scrapster.get_metrics_once(1000)
        print(f"sample {i}: cpu={m['cpu_usage_percent']:.2f}% mem_used={m['mem_used_bytes']}")
        time.sleep(0.1)


if __name__ == "__main__":
    main()


