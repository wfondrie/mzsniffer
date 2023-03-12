#!/usr/bin/env python3
from pathlib import Path
import sys
import json
import seaborn as sns
import matplotlib.pyplot as plt

def main():
    """The main function"""
    Path("img").mkdir(exist_ok=True)
    data = json.loads(sys.stdin.read())

    sns.set(style="ticks", context="paper")
    plt.figure(figsize=(8, 4))

    ret_times = data[0]["ret_times"]
    plt.xlabel("Retention Time (min)")
    plt.ylabel("Intensity")

    for poly in data[0]["polymers"]:
        plt.plot(ret_times, poly["xic"], label=poly["name"])
        plt.legend()

    plt.tight_layout()
    plt.savefig("img/example.png")


if __name__ == "__main__":
    main()
