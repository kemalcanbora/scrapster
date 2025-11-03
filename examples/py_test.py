import time

import scrapster


def main() -> None:
    # Collect 3 samples, 1s apart
    for i in range(3):
        m = scrapster.get_metrics_once(1000)
        print(m)
        time.sleep(0.1)


if __name__ == "__main__":
    main()


