window.BENCHMARK_DATA = {
  "lastUpdate": 1790291665357,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@zodl.com",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "54841311f4488bad7dc03bf5ec32ee912316dcbe",
          "message": "Merge pull request #943 from zcash/dw/deps\n\nUpdate ff, group, rand and other packages",
          "timestamp": "2026-09-25T00:08:25+01:00",
          "tree_id": "4ebb8f9fcb6f1970e76073c3f93d6be8b284a815",
          "url": "https://github.com/zcash/halo2/commit/54841311f4488bad7dc03bf5ec32ee912316dcbe"
        },
        "date": 1790291659627,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 70529919,
            "range": "± 3436755",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3737417,
            "range": "± 89738",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 127870787,
            "range": "± 2984989",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4168402,
            "range": "± 27577",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 170005440,
            "range": "± 744052",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4373350,
            "range": "± 28778",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 32419,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 105673,
            "range": "± 547",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 114279,
            "range": "± 1009",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 192603,
            "range": "± 1250",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 192699,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 107945,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 116430,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 194926,
            "range": "± 9238",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 194927,
            "range": "± 1839",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 226096,
            "range": "± 884",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 234471,
            "range": "± 676",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 312997,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 313019,
            "range": "± 1737",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}