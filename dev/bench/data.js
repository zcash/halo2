window.BENCHMARK_DATA = {
  "lastUpdate": 1791563447014,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f3354ec7e3d53852f55505d5d402ce792e1fc60d",
          "message": "Merge pull request #974 from zcash/fix-ci\n\nCI: Update apt cache before trying to install dev dependencies",
          "timestamp": "2026-10-09T10:25:02-06:00",
          "tree_id": "8abe3b62689e69963ca86a2d476b8f1f4db41227",
          "url": "https://github.com/zcash/halo2/commit/f3354ec7e3d53852f55505d5d402ce792e1fc60d"
        },
        "date": 1791563439204,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 70857325,
            "range": "± 441542",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3724308,
            "range": "± 60496",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 130807607,
            "range": "± 2391849",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4145146,
            "range": "± 64637",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 172364825,
            "range": "± 1218929",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4390692,
            "range": "± 28066",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 27365,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 101099,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 109175,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 183431,
            "range": "± 2758",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 183329,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 103211,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 111266,
            "range": "± 907",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 185464,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 185470,
            "range": "± 1183",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 215862,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 224102,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 298214,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 298216,
            "range": "± 3941",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}