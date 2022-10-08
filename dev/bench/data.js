window.BENCHMARK_DATA = {
  "lastUpdate": 1665232371727,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "def465de33b6c0023cedec018a3becb87bc56ca3",
          "message": "Merge pull request #658 from nalinbhardwaj/add-row-count\n\n[MockProver] Add row and col count to cost model",
          "timestamp": "2022-10-08T12:25:20+01:00",
          "tree_id": "1b19417bc09e5f32a817f42c3e13a79d5d706597",
          "url": "https://github.com/zcash/halo2/commit/def465de33b6c0023cedec018a3becb87bc56ca3"
        },
        "date": 1665232364433,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74922703,
            "range": "± 7447400",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3602977,
            "range": "± 196900",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 162295371,
            "range": "± 4842273",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4509940,
            "range": "± 236796",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 221682788,
            "range": "± 4622543",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4781532,
            "range": "± 109504",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46532,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 179320,
            "range": "± 14690",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 186077,
            "range": "± 461",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 299220,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 310317,
            "range": "± 15217",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 175109,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 189656,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 302837,
            "range": "± 4554",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 313426,
            "range": "± 10027",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 366370,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 380403,
            "range": "± 2273",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 491975,
            "range": "± 3228",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 489175,
            "range": "± 8931",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3665062,
            "range": "± 84060",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7183485,
            "range": "± 14647",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12601300,
            "range": "± 48230",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27871914,
            "range": "± 279153",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 49970642,
            "range": "± 581143",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 99241282,
            "range": "± 1235265",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8556,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10281,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18576,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23662,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33224,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 55483,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 121713,
            "range": "± 11924",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 230677,
            "range": "± 22282",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 479117,
            "range": "± 41366",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1017807,
            "range": "± 74216",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2100390,
            "range": "± 105589",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4571969,
            "range": "± 159201",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10005123,
            "range": "± 511880",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21759263,
            "range": "± 817759",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 47354284,
            "range": "± 1508898",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 110427644,
            "range": "± 4499340",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34158,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34249,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 185106903,
            "range": "± 15647752",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 396381930,
            "range": "± 11518840",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 848930558,
            "range": "± 2290371",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1894431656,
            "range": "± 69249970",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3972955334,
            "range": "± 107882023",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8512778601,
            "range": "± 111389674",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18003918358,
            "range": "± 205187566",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 38402277631,
            "range": "± 324893672",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 80981704289,
            "range": "± 418542376",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 118413046,
            "range": "± 9045302",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 209181300,
            "range": "± 9040370",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 372837226,
            "range": "± 13812846",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 638387643,
            "range": "± 2063994",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1221550274,
            "range": "± 33955778",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2244484470,
            "range": "± 26224578",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4440620716,
            "range": "± 119804219",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8452824336,
            "range": "± 184123179",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16236247519,
            "range": "± 318069644",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6150384,
            "range": "± 128017",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9872358,
            "range": "± 699595",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16022451,
            "range": "± 712498",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26989365,
            "range": "± 1500023",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 45180021,
            "range": "± 1327264",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 80176116,
            "range": "± 1633377",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 148190505,
            "range": "± 5406944",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 266670078,
            "range": "± 5301973",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 484689996,
            "range": "± 11877382",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}