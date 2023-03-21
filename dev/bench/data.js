window.BENCHMARK_DATA = {
  "lastUpdate": 1679435203237,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "ewillbefull@gmail.com",
            "name": "ebfull",
            "username": "ebfull"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cae65c4b60b54ead3ed285fb81adfee5f1b089c2",
          "message": "Merge pull request #741 from zcash/738-tracing-floor-planner\n\nCreate a `TracingFloorPlanner` for debugging purposes",
          "timestamp": "2023-03-21T14:48:15-06:00",
          "tree_id": "fb5ba4fafd0a1840b15f48146945f7bcadf44a25",
          "url": "https://github.com/zcash/halo2/commit/cae65c4b60b54ead3ed285fb81adfee5f1b089c2"
        },
        "date": 1679435195437,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 90722975,
            "range": "± 6146315",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4565839,
            "range": "± 431297",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 169267657,
            "range": "± 4778869",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5487430,
            "range": "± 469320",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 227331119,
            "range": "± 7012224",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6503977,
            "range": "± 597041",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38727,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 142102,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 154456,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 250574,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 250508,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144938,
            "range": "± 723",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 157375,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 253495,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 253373,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 303544,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 316016,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412272,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 412012,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3083905,
            "range": "± 17640",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5817894,
            "range": "± 1488",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10045379,
            "range": "± 25411",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22270703,
            "range": "± 238709",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41502264,
            "range": "± 479762",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 81472376,
            "range": "± 542895",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7223,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8639,
            "range": "± 2110",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14883,
            "range": "± 3547",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18792,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 25754,
            "range": "± 2886",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 42231,
            "range": "± 5345",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97267,
            "range": "± 14057",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 201877,
            "range": "± 35417",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 438236,
            "range": "± 64444",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 852316,
            "range": "± 105725",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1616970,
            "range": "± 204727",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3314644,
            "range": "± 354267",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 6986590,
            "range": "± 248537",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15222976,
            "range": "± 676094",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33442476,
            "range": "± 609491",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 75789165,
            "range": "± 881063",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28582,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28622,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 152385242,
            "range": "± 10932156",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 325705033,
            "range": "± 63870212",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 710228211,
            "range": "± 12780391",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1507434781,
            "range": "± 4950249",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3224859275,
            "range": "± 9852960",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6904439418,
            "range": "± 17169707",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14734684269,
            "range": "± 40841003",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31272477357,
            "range": "± 55142789",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66103345254,
            "range": "± 50650931",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 100981802,
            "range": "± 4354761",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 177518179,
            "range": "± 5612069",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 311921961,
            "range": "± 4311348",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 555480750,
            "range": "± 13453469",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1010198060,
            "range": "± 16048770",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1876489808,
            "range": "± 11817391",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3567600839,
            "range": "± 16762341",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6908085325,
            "range": "± 17132005",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13262263794,
            "range": "± 107750973",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5060826,
            "range": "± 409334",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7787460,
            "range": "± 586765",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12661714,
            "range": "± 903659",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21584946,
            "range": "± 1484832",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 36816563,
            "range": "± 1974379",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66048203,
            "range": "± 4339258",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 117971780,
            "range": "± 4499993",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 220044887,
            "range": "± 12895042",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 411753884,
            "range": "± 6809579",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}