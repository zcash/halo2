window.BENCHMARK_DATA = {
  "lastUpdate": 1674251638707,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d771a48f686af0a65f5171905b8b855e55774bd1",
          "message": "Merge pull request #718 from HollowMan6/main\n\nFix book deploying CI and add pdf uploading",
          "timestamp": "2023-01-20T20:52:28Z",
          "tree_id": "3d7911ccfaf47965579d79c55ccef2ec11fcefc7",
          "url": "https://github.com/zcash/halo2/commit/d771a48f686af0a65f5171905b8b855e55774bd1"
        },
        "date": 1674251631062,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 73740172,
            "range": "± 7239544",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3479011,
            "range": "± 385453",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 147544183,
            "range": "± 7678784",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4066359,
            "range": "± 495993",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 195531850,
            "range": "± 6891199",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5010145,
            "range": "± 711265",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 40321,
            "range": "± 2740",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 145920,
            "range": "± 8881",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 155556,
            "range": "± 11014",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 263683,
            "range": "± 17354",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 277370,
            "range": "± 14344",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 150291,
            "range": "± 9340",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 171512,
            "range": "± 8490",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 262407,
            "range": "± 13230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 264388,
            "range": "± 14965",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 315738,
            "range": "± 17504",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 324113,
            "range": "± 21028",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 433228,
            "range": "± 23016",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 441956,
            "range": "± 24292",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3140291,
            "range": "± 164993",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6537749,
            "range": "± 374964",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10855090,
            "range": "± 214710",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23628065,
            "range": "± 501829",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43009943,
            "range": "± 880961",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 88379517,
            "range": "± 3023147",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8218,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9748,
            "range": "± 2415",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16314,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19824,
            "range": "± 1048",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27902,
            "range": "± 2888",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46581,
            "range": "± 6912",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 109022,
            "range": "± 15917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 213515,
            "range": "± 42581",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 494755,
            "range": "± 70705",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 899636,
            "range": "± 107832",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1701245,
            "range": "± 157789",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3690720,
            "range": "± 400380",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7575680,
            "range": "± 469238",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 16300122,
            "range": "± 964245",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 36855055,
            "range": "± 1655986",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 83532428,
            "range": "± 3454172",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 29457,
            "range": "± 1710",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 30811,
            "range": "± 1741",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 169113247,
            "range": "± 10123170",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 362495011,
            "range": "± 8076985",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 770797845,
            "range": "± 28196529",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1671149332,
            "range": "± 44786589",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3557550657,
            "range": "± 92135215",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7682556278,
            "range": "± 166439554",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 15977101922,
            "range": "± 345761459",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 33977038208,
            "range": "± 321198572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 72034807458,
            "range": "± 648556738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 113431328,
            "range": "± 5393781",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 198427517,
            "range": "± 7146797",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 337500037,
            "range": "± 8411265",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 613121892,
            "range": "± 27138008",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1084167014,
            "range": "± 39867484",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2047570747,
            "range": "± 39786223",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3884807430,
            "range": "± 136724103",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7422166022,
            "range": "± 96130193",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14702661892,
            "range": "± 270764155",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5703017,
            "range": "± 601135",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8656962,
            "range": "± 988387",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 13960924,
            "range": "± 1336306",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 23931492,
            "range": "± 2009082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 41693090,
            "range": "± 2826494",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 73471090,
            "range": "± 4079080",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 132452801,
            "range": "± 7739169",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 245556147,
            "range": "± 19177343",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 442784182,
            "range": "± 16503100",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}