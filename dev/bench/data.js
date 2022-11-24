window.BENCHMARK_DATA = {
  "lastUpdate": 1669259385102,
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
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "distinct": true,
          "id": "0ba0e40b874129e586fec0336c08011ef7e049a3",
          "message": "CI: Avoid testing against MSRV with test-dev-graph feature flag\n\nWe only need it for generating images of halo2_gadgets chips, and its\ntransitive dependencies have bumped MSRV in point releases.",
          "timestamp": "2022-11-24T01:56:54Z",
          "tree_id": "2f32fcd9ef72a5ac8a509e640656285eeacb6f0a",
          "url": "https://github.com/zcash/halo2/commit/0ba0e40b874129e586fec0336c08011ef7e049a3"
        },
        "date": 1669259377832,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 73712399,
            "range": "± 8268325",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3812091,
            "range": "± 246017",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 153831764,
            "range": "± 5579720",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4633189,
            "range": "± 209585",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 224579710,
            "range": "± 10936617",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5283400,
            "range": "± 362813",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 55423,
            "range": "± 1833",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 182800,
            "range": "± 4957",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 198099,
            "range": "± 5646",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 327340,
            "range": "± 9406",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 328691,
            "range": "± 16833",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 187115,
            "range": "± 5895",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 203912,
            "range": "± 15915",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 330008,
            "range": "± 12219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 327529,
            "range": "± 18905",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 398737,
            "range": "± 16661",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 412351,
            "range": "± 9291",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 502865,
            "range": "± 23784",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 486483,
            "range": "± 19573",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3734541,
            "range": "± 182885",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7362077,
            "range": "± 177580",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12796272,
            "range": "± 358761",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 25289890,
            "range": "± 625661",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 54493475,
            "range": "± 1826929",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 107618049,
            "range": "± 1181413",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8148,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9649,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18115,
            "range": "± 1078",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21458,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33453,
            "range": "± 1599",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 58949,
            "range": "± 4463",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 124014,
            "range": "± 9763",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 252265,
            "range": "± 31531",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 500145,
            "range": "± 35477",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1060500,
            "range": "± 60029",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2298256,
            "range": "± 111953",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4986855,
            "range": "± 142683",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10615229,
            "range": "± 350266",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22447160,
            "range": "± 889614",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 48436979,
            "range": "± 1873917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 119307012,
            "range": "± 15050354",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 37265,
            "range": "± 1845",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 37203,
            "range": "± 1869",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 211096194,
            "range": "± 3363270",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 422831789,
            "range": "± 21755944",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 931086096,
            "range": "± 32133847",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1991118647,
            "range": "± 30410181",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4255674801,
            "range": "± 45597595",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9036782489,
            "range": "± 42365992",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 19943693508,
            "range": "± 158893298",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 42818447755,
            "range": "± 269136383",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 90082282819,
            "range": "± 1936418332",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 122989690,
            "range": "± 2707407",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 221960265,
            "range": "± 6215525",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 391813031,
            "range": "± 9340468",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 715881328,
            "range": "± 10906971",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1334675646,
            "range": "± 8853015",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2491743759,
            "range": "± 36445898",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4563398206,
            "range": "± 110255246",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8902199880,
            "range": "± 74675613",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17366978256,
            "range": "± 291702539",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6696913,
            "range": "± 293822",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10414785,
            "range": "± 672369",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16608758,
            "range": "± 1016006",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 28497625,
            "range": "± 1901471",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 50563534,
            "range": "± 1907551",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 89906687,
            "range": "± 2852192",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 143764977,
            "range": "± 7356938",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 269877860,
            "range": "± 9355335",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 486802905,
            "range": "± 22671905",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}