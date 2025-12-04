window.BENCHMARK_DATA = {
  "lastUpdate": 1764877976727,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "61cce5952c3a84b6c76cbc3e69561768a128a8f7",
          "message": "Merge pull request #796 from ashWhiteHat/feature/msm-optimization\n\nOptimize Msm",
          "timestamp": "2025-12-04T19:42:58Z",
          "tree_id": "96d11d63d4e35ca41590229006b375641a632793",
          "url": "https://github.com/zcash/halo2/commit/61cce5952c3a84b6c76cbc3e69561768a128a8f7"
        },
        "date": 1764877970879,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 78044482,
            "range": "± 1484508",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4276919,
            "range": "± 28246",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 143988698,
            "range": "± 2829985",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4743312,
            "range": "± 31985",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 196067232,
            "range": "± 1769952",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4996161,
            "range": "± 24788",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31407,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133091,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145795,
            "range": "± 2657",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 236714,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 236764,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135614,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148541,
            "range": "± 10818",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 239492,
            "range": "± 826",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 239338,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284197,
            "range": "± 11835",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296766,
            "range": "± 6201",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387611,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387655,
            "range": "± 1703",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}